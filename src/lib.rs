use std::{ffi::{c_char, c_ulong}, ptr::NonNull, thread, time};
use std::convert::Into;
use std::mem::MaybeUninit;
use tokio;

use nexus_rs::raw_structs::{AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, ELogLevel, LPVOID};
use windows::{
    core::s,
    Win32::{
        Foundation::{HINSTANCE, HMODULE},
        System::SystemServices,
    },
};

use crate::rich_presence_core::NexusRichPresence;

mod rich_presence_core;

static mut HANDLE: Option<HMODULE> = None;
static DISCORD_APP_ID_I64: i64 = 1180951923722039316;

#[no_mangle]
unsafe extern "C" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: c_ulong,
    _lpv_reserveded: LPVOID,
) -> bool {
    match fdw_reason {
        SystemServices::DLL_PROCESS_ATTACH => {
            HANDLE = Some(hinst_dll.into());
        }
        _ => {}
    }
    true
}

static mut NRP: MaybeUninit<NexusRichPresence> = MaybeUninit::uninit();

#[tokio::main]
async unsafe extern "C" fn load(a_api: *mut AddonAPI) {
    NRP.write(NexusRichPresence::new(*a_api, DISCORD_APP_ID_I64).await);

    NRP.assume_init_mut().start().await;

    NRP.assume_init_mut().log(ELogLevel::INFO, format!("Loaded!\0"));
}

#[tokio::main]
async unsafe extern "C" fn unload() {
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
            NonNull::new_unchecked(s!("https://github.com/mythwright/nexus-rich-discord").0 as _)
        }),
    };

    &AD as *const _ as _
}