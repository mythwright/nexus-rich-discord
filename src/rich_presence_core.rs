use log::{error, info};
use nexus_rs::raw_structs::{AddonAPI, ELogLevel};

pub struct NexusRichPresence {
    pub api: Option<AddonAPI>,
    pub discord: Option<discord_sdk::Discord>,
    pub user: Option<discord_sdk::user::User>,
}

unsafe impl Send for NexusRichPresence {}

impl NexusRichPresence {
    pub async unsafe fn start(&mut self) {
        if self.discord.is_none() {
            return
        }

        info!("Discord is verified");

        let rp = discord_sdk::activity::ActivityBuilder::default()
            .details("Starting")
            .state("And Started")
            .button(discord_sdk::activity::Button{
                label: "github".to_owned(),
                url: "https://github.com".to_owned(),
            });

        let d = self.discord.as_mut().unwrap();
        match d.update_activity(rp).await {
            _ => {
                self.log(ELogLevel::INFO, "Updated Activity\0".to_string())
            }
        }

        self.log(ELogLevel::INFO, "Holding\0".to_string());
    }

    pub async unsafe fn unload(self) {
        if self.discord.is_none() {
            if self.api.is_some() {
                self.log(ELogLevel::CRITICAL, "No Discord Client has been loaded to disconnect\0".to_string())
            }
            return
        }
        if self.api.is_some() {
            self.log(ELogLevel::INFO, "Disconnected Discord Client\0".to_string())
        }
        self.discord.unwrap().disconnect().await;
    }
    pub unsafe fn log(&self,level: ELogLevel, s: String) {
        if self.api.is_none() {
            return
        }
        (self.api.unwrap().log)(
            level,
            s.as_ptr() as _,
        );
    }

    pub async unsafe fn set_discord(&mut self, discord_app_id: i64) {
        let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
            error!("encountered an error in discord {}", err);
        }));

        let mut user = wheel.user();

        let disc = discord_sdk::Discord::new(
            discord_sdk::DiscordApp::PlainId(discord_app_id),
            discord_sdk::Subscriptions::ACTIVITY,
            Box::new(handler)).expect("failed to create discord client");

        info!("waiting for discord handshake");
        self.log(ELogLevel::INFO, "waiting for discord handshake\0".to_string());
        user.0.changed().await.unwrap();

        let user = match &*user.0.borrow() {
            discord_sdk::wheel::UserState::Connected(user) => user.clone(),
            discord_sdk::wheel::UserState::Disconnected(err) => {
                error!("failed to connect to discord: {}", err);
                return;
            },
        };

        info!("connected to Discord as user {:#?}", user);
        self.log(ELogLevel::INFO, format!("connected to Discord as user {:#?}\0", user));
        self.discord = Some(disc);
    }

    pub async unsafe fn set_api(&mut self, api: AddonAPI) {
        self.api = Some(api);
    }

    pub unsafe fn new_blank() -> NexusRichPresence {
        NexusRichPresence {
            api: None,
            discord: None,
            user: None,
        }
    }

    pub async unsafe fn new(api: AddonAPI, discord_app_id: i64) -> NexusRichPresence {
        let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|_err| {
        }));

        let disc = discord_sdk::Discord::new(
            discord_sdk::DiscordApp::PlainId(discord_app_id),
            discord_sdk::Subscriptions::ACTIVITY,
            Box::new(handler)).unwrap();


        let mut user = wheel.user();

        user.0.changed().await.unwrap();

        let user = match &*user.0.borrow() {
            discord_sdk::wheel::UserState::Connected(user) => user.clone(),
            discord_sdk::wheel::UserState::Disconnected(_err) => {
                panic!("failed to connect");
            },
        };

        NexusRichPresence {
            api: Some(api),
            discord: Some(disc),
            user: Some(user),
        }
    }
}