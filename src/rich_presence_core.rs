use std::sync::Arc;
use std::time::Duration;
use nexus_rs::raw_structs::ELogLevel;
use tokio::runtime;
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;
use crate::API;

static mut RUNTIME: OnceCell<runtime::Runtime> = OnceCell::const_new();


pub struct NexusRichPresence {
    pub discord: OnceCell<discord_sdk::Discord>,
    pub shutdown: OnceCell<bool>,
    discord_id: i64,
}

unsafe impl Send for NexusRichPresence {}

impl NexusRichPresence {
    pub unsafe fn start(self: Arc<Self>) -> JoinHandle<()> {
        RUNTIME.set(runtime::Runtime::new().unwrap()).expect("TODO: panic message");
        RUNTIME.get().unwrap().spawn(async move {
            let s = self.shutdown.get_or_init(|| async {false}).await;
            while !s {
                self.start_discord().await;
                self.update_act("Sitting at Character Select".to_string(), "AFK".to_string()).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
    }

    pub async fn start_discord(&self) {
        self.discord.get_or_init(|| async {
            let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|_err| {}));
            let mut user = wheel.user();

            let disc = discord_sdk::Discord::new(
                discord_sdk::DiscordApp::PlainId(self.discord_id),
                discord_sdk::Subscriptions::ACTIVITY,
                Box::new(handler)).unwrap();

            self.log(ELogLevel::INFO, "waiting for Discord handshake".to_string());
            user.0.changed().await.unwrap();

            match &*user.0.borrow() {
                discord_sdk::wheel::UserState::Connected(user) => user.clone(),
                discord_sdk::wheel::UserState::Disconnected(err) => panic!("failed to connect to Discord: {}", err),
            };

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

    pub async unsafe fn unload(self) {
        // if self.discord.is_none() {
        //     if self.api.is_some() {
        //         self.log(ELogLevel::CRITICAL, "No Discord Client has been loaded to disconnect\0".to_string())
        //     }
        //     return
        // }
        // if self.api.is_some() {
        //     self.log(ELogLevel::INFO, "Disconnected Discord Client\0".to_string())
        // }
        // self.discord.unwrap().disconnect().await;
    }
    pub fn log(&self,level: ELogLevel, s: String) {
        unsafe {
            let api = API.assume_init();
            (api.log)(
                level,
                (s+"\0").as_ptr() as _,
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