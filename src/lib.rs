use std::{ffi::{c_char, c_ulong}, mem::MaybeUninit, ptr::NonNull};
use std::convert::Into;
use nexus_rs::raw_structs::{
    AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, ELogLevel, LPVOID,
};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use windows::{
    core::s,
    Win32:: {
        Foundation::{HINSTANCE, HMODULE},
        System::SystemServices
    }
};
use once_cell::sync::Lazy;
use bitmask_enum::bitmask;
use serde::Deserialize;

mod rich_presence_core;

static mut API: MaybeUninit<&'static AddonAPI> = MaybeUninit::uninit();
static mut HANDLE: Option<HMODULE> = None;
static DISCORD_APP_ID: &str = "";

#[no_mangle]
unsafe extern "C" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: c_ulong,
    _lpv_reserveded: LPVOID,
) -> bool {
    match fdw_reason{
        SystemServices::DLL_PROCESS_ATTACH => {
            HANDLE = Some(hinst_dll.into());
        }
        _ => {}
    }
    true
}

#[bitmask(u32)]
enum UiState {
    IsMapOpen = 1 << 0,
    IsCompassTopRight = 1 << 1,
    DoesCompassHaveRotationEnabled = 1 << 2,
    GameHasFocus = 1 << 3,
    IsInCompetitiveGameMode = 1 << 4,
    TextboxHasFocus = 1 << 5,
    IsInCombat = 1 << 6,
}

#[derive(Deserialize, Debug)]
struct Identity {
    pub name: String,
    pub profession: u8,
    pub spec: u16,
    pub race: u16,
    pub map_id: usize,
    pub world_id: usize,
    pub team_color_id: usize,
    pub commander: bool,
    pub fov: f64,
    pub uisz: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct MumbleContext {
    pub server_address: [u8; 28],
    pub map_id: u32,
    pub map_type: u32,
    pub shard_id: u32,
    pub instance: u32,
    pub build_id: u32,
    pub ui_state: UiState,
    pub compass_width: u16,
    pub compass_height: u16,
    pub compass_rotation: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub map_center_x: f32,
    pub map_center_y: f32,
    pub map_scale: f32,
    pub process_id: u32,
    pub mount_index: u8,
}

#[derive(Debug)]
#[repr(C)]
struct MumbleLinkData {
    pub ui_version: u32,
    pub ui_tick: u32,
    pub identity: Identity,
    pub context: MumbleContext,
}
static mut MUMBLE_DATA: Option<&'static MumbleLinkData> = None;


unsafe extern "C" fn load(a_api: *mut AddonAPI) {
    let mut nrp = rich_presence_core::NexusRichPresence::new(*a_api, DISCORD_APP_ID);
    nrp.log(ELogLevel::INFO, format!("Loaded Discord Rich Presence\0"));

    match nrp.discord.connect() {
        Err(error) => {
            nrp.log(ELogLevel::CRITICAL, format!("{:?}\0", error))
        }
        _ => {
            nrp.log(ELogLevel::INFO, format!("Successful connected\0"));
        }
    }
    //
    // if loaded {
    //     match DISCORD_CLIENT.set_activity(activity::Activity::new()
    //         .state("AFKing")
    //         .details("Testing a plugin")) {
    //         Err(error) => {
    //             let s = format!("{:?}\0", error);
    //             (API.assume_init().log)(ELogLevel::CRITICAL, s.as_ptr() as _)
    //         },
    //         _ => {
    //             let s = format!("Successful sent set activity\0");
    //             (API.assume_init().log)(ELogLevel::INFO, s.as_ptr() as _)
    //         }
    //     }
    // }
}


unsafe extern "C" fn unload() {
    // match DISCORD_CLIENT.close() {
    //     Err(_) => (API.assume_init().log)(ELogLevel::CRITICAL, s!("Failed to close client").0 as _),
    //     _ => {
    //         (API.assume_init().log)(ELogLevel::INFO, s!("Rich Presence Unload Successful").0 as _)
    //     }
    // }
}

#[no_mangle]
pub extern "C" fn GetAddonDef() -> *mut AddonDefinition {
    static AD: AddonDefinition = AddonDefinition {
        signature: -32409,
        apiversion: nexus_rs::raw_structs::NEXUS_API_VERSION,
        name: b"Nexus Rich Presence\0".as_ptr() as *const c_char,
        version: AddonVersion {
            major: 0,
            minor: 0,
            build: 1,
            revision: 0,
        },
        author: b"Zyian\0".as_ptr() as *const c_char,
        description: b"A Discord Rich Presence addon for showing your current status in game\0".as_ptr() as *const c_char,
        load,
        unload: Some(unsafe { NonNull::new_unchecked(unload as _) }),
        flags: EAddonFlags::None,
        provider: nexus_rs::raw_structs::EUpdateProvider::GitHub,
        update_link: Some(unsafe {
            NonNull::new_unchecked(s!("https://github.com/mythwright/nexus-rich-presence").0 as _)
        })
    };

    &AD as *const _ as _
}