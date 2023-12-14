use nexus_rs::raw_structs::{AddonAPI, ELogLevel};

pub struct NexusRichPresence {
    pub api: Option<AddonAPI>,
    pub discord: discord_sdk::Discord,
    pub user: discord_sdk::user::User,
}


impl NexusRichPresence {
    pub async unsafe fn start(&mut self) {
        let rp = discord_sdk::activity::ActivityBuilder::default()
            .details("Starting")
            .state("And Started")
            .button(discord_sdk::activity::Button{
                label: "github".to_owned(),
                url: "https://github.com".to_owned(),
            });

        match self.discord.update_activity(rp).await {
            Err(err) => {
                self.log(ELogLevel::CRITICAL, format!("Unknown Err {}!\0", err));
            },
            _ => {
                self.log(ELogLevel::INFO, format!("Sent!\0"));
            }
        }

        // match self.discord.connect() {
        //     Err(e) => {
        //         self.log(ELogLevel::CRITICAL, format!("No"))
        //     }
        //     _ => {}
        // }
        //
        // match self.discord.set_activity(activity::Activity::new().state("AFKING")) {
        //     Err(e) => {
        //         self.log(ELogLevel::CRITICAL, format!("No"))
        //     }
        //     _ => {}
        // }
    }

    pub async unsafe fn unload(self) {
        let b = self.discord;
        b.disconnect().await;
    }
    pub unsafe fn log(&self,level: ELogLevel, s: String) {
        (self.api.unwrap().log)(
            level,
            s.as_ptr() as _,
        );
    }
    pub async unsafe fn new(api: AddonAPI, discord_app_id: i64) -> NexusRichPresence {
        let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
        }));

        let disc = discord_sdk::Discord::new(
            discord_sdk::DiscordApp::PlainId(discord_app_id),
            discord_sdk::Subscriptions::ACTIVITY,
            Box::new(handler)).unwrap();


        let mut user = wheel.user();

        user.0.changed().await.unwrap();

        let user = match &*user.0.borrow() {
            discord_sdk::wheel::UserState::Connected(user) => user.clone(),
            discord_sdk::wheel::UserState::Disconnected(err) => {
                panic!("failed to connect");
            },
        };

        NexusRichPresence {
            api: Some(api),
            discord: disc,
            user,
        }
    }
}