use std::{ffi::{c_char, c_ulong}, ptr::NonNull};
use std::convert::Into;
use std::mem::MaybeUninit;
use tokio;

use nexus_rs::raw_structs::{AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, LPVOID};
use once_cell::sync::OnceCell;
use tokio::task::JoinHandle;
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
static mut THREADS: OnceCell<Vec<JoinHandle<()>>> = OnceCell::new();
static mut API: MaybeUninit<&'static AddonAPI> = MaybeUninit::uninit();
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

unsafe extern "C" fn load(a_api: *mut AddonAPI) {
    API.write(&*a_api);
    THREADS.set(Vec::new()).expect("TODO: panic message");

    let n = NexusRichPresence::new(DISCORD_APP_ID_I64);
    let q= n.start();
    THREADS.get_mut().unwrap().push(q);
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