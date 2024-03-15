use crate::mumble_data::{CMumbleLinkData, Identity, MumbleContext, Position};
use crate::rich_presence_core::NexusRichPresence;
use nexus_rs::raw_structs::{
    AddonAPI, AddonDefinition, AddonVersion, EAddonFlags, ELogLevel, LPVOID,
};
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
use std::ffi::{c_void, CStr};
use tokio;
use tokio::runtime::Builder;
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
static mut AAA: OnceCell<NexusRichPresence> = OnceCell::new();

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
    let mumble_data =
        (API.assume_init().get_resource)(s!("DL_MUMBLE_LINK").0 as _) as *const CMumbleLinkData;
    if mumble_data.is_null() {
        panic!("no mumble")
    }


    let file = File::create("debug_discord.log");
    let file = match file {
        Ok(file) => file,
        Err(_) => panic!("Error"),
    };
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(Arc::new(file))
        .with_ansi(false)
        .init();

    THREADS.set(Vec::new()).expect("TODO: panic message");

    THREADS.get_mut().unwrap().push(spawn(|| {
        let _ = AAA.set(NexusRichPresence::new(DISCORD_APP_ID_I64));
        (API.assume_init().subscribe_event)(s!("EV_MUMBLE_IDENTITY_UPDATED").0 as _, event_process);
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(AAA.get_mut().unwrap().start());
    }));
}

pub unsafe extern "C" fn event_process(event_args: *mut c_void) {
    let e = event_args as *const Identity;
    let ee = *e;

    let api = API.assume_init();
    (api.log)(
        ELogLevel::DEBUG,
        (format!("{:?}", ee).to_string() + "\0").as_ptr() as _,
    );

    if AAA.get().is_none() {
        return;
    }

    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(runtime.spawn(AAA.get_mut().unwrap().process_event(ee))).unwrap();
}

unsafe extern "C" fn unload() {
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();
    let h = runtime.spawn(AAA.get_mut().unwrap().shutdown());
    runtime.block_on(h).unwrap();
    for t in THREADS.take().unwrap() {
        let _ = t.join();
    }
    let q = AAA.take();
    if q.is_none() {
        let api = API.assume_init();
        (api.log)(
            ELogLevel::DEBUG,
            ("Unable to obtain inner value".to_string() + "\0").as_ptr() as _,
        );
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
