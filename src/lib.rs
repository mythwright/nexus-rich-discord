use crate::rich_presence_core::NexusRichPresence;
use nexus_rs::raw_structs::{AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, LPVOID};
use once_cell::sync::OnceCell;
use std::{
    convert::Into,
    ffi::{c_char, c_ulong},
    fs::File,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::Arc,
    thread::{spawn, JoinHandle},
};
use tokio;
use windows::{
    core::s,
    Win32::{
        Foundation::{HINSTANCE, HMODULE},
        System::SystemServices,
    },
};

mod mumble_data;
mod rich_presence_core;

static mut HANDLE: Option<HMODULE> = None;
static mut THREADS: OnceCell<Vec<JoinHandle<()>>> = OnceCell::new();
static mut API: MaybeUninit<&'static AddonAPI> = MaybeUninit::uninit();
static DISCORD_APP_ID_I64: i64 = 1180951923722039316;
static mut AAA: OnceCell<Arc<NexusRichPresence>> = OnceCell::new();

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
    let mumbledata = (API.assume_init().get_resource)(s!("DL_MUMBLE_LINK").0 as _);
    if mumbledata.is_null() {
        panic!("no mumble")
    } else {
    }

    let file = File::create("debug_discord.log");
    let file = match file {
        Ok(file) => file,
        Err(error) => panic!("Error"),
    };
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(Arc::new(file))
        .with_ansi(false)
        .init();

    THREADS.set(Vec::new()).expect("TODO: panic message");

    THREADS.get_mut().unwrap().push(spawn(|| {
        let n = NexusRichPresence::new(DISCORD_APP_ID_I64);
        let _ = AAA.set(n);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(AAA.get_mut().unwrap().start());
    }));
}

unsafe extern "C" fn unload() {
    // TODO: Figure out how to send the shutdown signal here
    // Causes crashes when trying to unload due to forever waiting on the threads to join
    for t in THREADS.take().unwrap() {
        let _ = t.join();
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
        description: b"A Discord Rich Presence addon for showing your current status in game\0"
            .as_ptr() as *const c_char,
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
