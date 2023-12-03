use std::{ffi::{c_char, c_ulong}, mem::MaybeUninit, ptr, ptr::NonNull};
use std::convert::Into;
use std::ffi::c_void;
use nexus_rs::raw_structs::{
    AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, ELogLevel, LPVOID,
};
use arcdps_imgui::{
    self,
    sys::{igSetAllocatorFunctions, igSetCurrentContext},
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

static mut DISCORD_CLIENT: Lazy<DiscordIpcClient> = Lazy::new(|| DiscordIpcClient::new(DISCORD_APP_ID).unwrap());


unsafe extern "C" fn load(a_api: *mut AddonAPI) {
    let api = &*a_api;
    API.write(&api);

    igSetCurrentContext(api.imgui_context);
    igSetAllocatorFunctions(
        Some(api.imgui_malloc),
        Some(api.imgui_free),
        ptr::null::<c_void>() as *mut _,
    );

    (api.log)(ELogLevel::INFO, s!("Loaded Discord Rich Presence").0 as _);
    DISCORD_CLIENT.connect().unwrap();

    match DISCORD_CLIENT.set_activity(activity::Activity::new()
        .state("AFKing")
        .details("Testing a plugin")) {
        Err(error) => {
            let s = format!("{:?}\0", error);
            (API.assume_init().log)(ELogLevel::CRITICAL, s.as_ptr() as _)
        },
        _ => {}
    }
}


unsafe extern "C" fn unload() {
    match DISCORD_CLIENT.close() {
        Err(_) => (API.assume_init().log)(ELogLevel::CRITICAL, s!("Failed to close client").0 as _),
        _ => {}
    }
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