use discord_rich_presence::DiscordIpcClient;
use nexus_rs::raw_structs::{AddonAPI, ELogLevel};

pub struct NexusRichPresence {
    pub api: AddonAPI,
    pub discord: DiscordIpcClient,
}

impl NexusRichPresence {
    pub fn start(self) {

    }

    pub unsafe fn log(&self,level: ELogLevel, s: String) {
        (self.api.log)(
            level,
            s.as_ptr() as _,
        );
    }
    pub fn new(api: AddonAPI, discord_app_id: &str) -> NexusRichPresence {
        let discord = DiscordIpcClient::new(discord_app_id).unwrap();
        NexusRichPresence {
            api,
            discord
        }
    }
}