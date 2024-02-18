use std::sync::{Arc};
use std::time::Duration;
use nexus_rs::raw_structs::ELogLevel;
use tokio::sync::OnceCell;
use tokio::sync::RwLock;
use crate::API;


pub struct NexusRichPresence {
    pub discord: OnceCell<discord_sdk::Discord>,
    pub shutdown: RwLock<bool>,
    discord_id: i64,
}

impl NexusRichPresence {
    pub async unsafe fn start(self: &mut Arc<Self>) {
        self.log(ELogLevel::DEBUG, "Starting Discord...".to_string());
        self.start_discord().await;
        if !self.discord.initialized() {
            self.log(ELogLevel::CRITICAL, "Discord not initialized".to_string());
        }
        self.log(ELogLevel::DEBUG, "Done Waiting for Discord...".to_string());
        loop {
            {
                let shut = self.shutdown.read().await;
                if !*shut {
                    self.log(ELogLevel::DEBUG, "Updating Discord...".to_string());
                    self.update_act("Sitting at Character Select".to_string(), "AFK".to_string()).await;
                }
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    pub async fn start_discord(&self) {
        let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
            tracing::error!(error = ?err, "encountered an error");
        }));
        let mut user = wheel.user();

        self.log(ELogLevel::DEBUG, "Creating Discord SDK...".to_string());
        let disc = discord_sdk::Discord::new(
            discord_sdk::DiscordApp::PlainId(self.discord_id),
            discord_sdk::Subscriptions::ACTIVITY,
            Box::new(handler)).expect("unable to create discord client");

        self.log(ELogLevel::DEBUG, "Waiting for Discord handshake".to_string());
        user.0.changed().await.unwrap();

        self.log(ELogLevel::DEBUG, "Got Discord Connection".to_string());
        match &*user.0.borrow() {
            discord_sdk::wheel::UserState::Connected(user) => user.clone(),
            discord_sdk::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {}", err),
        };
        self.discord.get_or_init(|| async {
            disc
        }).await;
    }

    pub async fn update_act(&self, details: String, state: String) {
        let rp = discord_sdk::activity::ActivityBuilder::default()
            .details(details)
            .state(state);

        match self.discord.get().unwrap().update_activity(rp).await {
            Err(e) => {
                self.log(ELogLevel::CRITICAL, format!("Some error updating {}", e))
            }
            _ => {
                // self.log(ELogLevel::INFO, "Updated Activity".to_string())
            }
        }
    }

    pub async fn unload(mut self) {
        {
            let mut shut = self.shutdown.write().await;
            *shut = true;
        }


        let d = self.discord.take().unwrap();
        let _ = d.disconnect();
        self.log(ELogLevel::INFO, "Discord disconnected".to_string())
    }
    pub fn log(&self, level: ELogLevel, s: String) {
        unsafe {
            let api = API.assume_init();
            (api.log)(
                level,
                (s + "\0").as_ptr() as _,
            );
        }
    }

    pub fn new(discord_app_id: i64) -> Arc<Self> {
        Arc::new(Self {
            discord: OnceCell::new(),
            discord_id: discord_app_id,
            shutdown: Default::default(),
        })
    }
}