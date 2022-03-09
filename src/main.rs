#![windows_subsystem = "windows"]

use std::collections::HashMap;
use std::env::args;
use std::ptr::null;
use directories::{ProjectDirs};
use hocon::HoconLoader;
use serde::Deserialize;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError};
use windows::Win32::System::Threading::{CreateProcessWithLogonW, LOGON_WITH_PROFILE, PROCESS_INFORMATION};

#[derive(Deserialize, Debug, Default)]
struct Config {
    #[serde(default)]
    credentials: HashMap<String, String>,
}

fn main() {
    let dir = ProjectDirs::from("ax", "nulldev", "runwithcreds")
        .expect("Failed to determine project dirs!");

    let config = dir.config_dir().join("creds.conf");
    println!("Using config: {config:?}");

    let config: Config = HoconLoader::new()
        .load_file(config)
        .expect("Failed to load config!")
        .resolve()
        .expect("Failed to deserialize config!");

    let mut args_str = args().skip(1);
    let user = args_str.next().expect("No user supplied!");
    let prog = args_str.next().expect("Program not supplied!");

    let password = config.credentials.get(&user).expect("No password found for user!");

    let mut process_info = PROCESS_INFORMATION::default();
    let result = unsafe {
        CreateProcessWithLogonW(
            user,
            ".",
            password.clone(),
            LOGON_WITH_PROFILE,
            prog,
            PWSTR::default(),
            0,
            null(),
            PCWSTR::default(),
            null(),
            &mut process_info
        )
    };

    if !result.as_bool() {
        let exception = unsafe { GetLastError() };
        panic!("Failed to launch new process: {:?}", exception);
    }

    unsafe {
        CloseHandle(process_info.hProcess);
        CloseHandle(process_info.hThread);
    }
}
