use std::{ffi::{c_char, c_ulong}, ptr::NonNull};
use std::convert::Into;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use tokio;

use nexus_rs::raw_structs::{AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, ELogLevel, LPVOID};
use once_cell::sync::OnceCell;
use windows::{
    core::s,
    Win32::{
        Foundation::{HINSTANCE, HMODULE},
        System::SystemServices,
        UI::WindowsAndMessaging::MessageBoxA
    },
};
use windows::Win32::Foundation::HWND;

use crate::rich_presence_core::NexusRichPresence;

pub mod rich_presence_core;

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

// lazy_static!{
//     static ref NRP: Arc<Mutex<NexusRichPresence>> = Arc::new(Mutex::new(unsafe {NexusRichPresence::new_blank()}));
// }
// static mut NRP: MaybeUninit<NexusRichPresence> = MaybeUninit::uninit();
static mut NRP: OnceCell<Arc<Mutex<NexusRichPresence>>> = OnceCell::new();


#[tokio::main]
async unsafe extern "C" fn load(a_api: *mut AddonAPI) {
    let n = NRP.get_or_init(|| Arc::new(Mutex::new(unsafe {NexusRichPresence::new_blank()})));
    n.lock().unwrap().set_api(*a_api).await;
    n.lock().unwrap().set_discord(DISCORD_APP_ID_I64).await;
    n.lock().unwrap().start().await;

    n.lock().unwrap().log(ELogLevel::INFO, "Nexus Rich Presence has been loaded.\0".to_string());
}

#[tokio::main]
async unsafe extern "C" fn unload() {
    match Arc::into_inner(NRP.take().unwrap()) {
        None => {
            MessageBoxA(HWND(0),
                        s!("Cannot into_inner of Mutex"),
                        s!("Nexus Rich Presence"),
                        Default::default()
            );

        }
        Some(n) => {
            n.into_inner().unwrap().unload().await;
        }
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
            NonNull::new_unchecked(s!("https://github.com/mythwright/nexus-rich-discord").0 as _)
        }),
    };

    &AD as *const _ as _
}