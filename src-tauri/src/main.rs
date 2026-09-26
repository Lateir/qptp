// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  #[cfg(windows)]
  {
    let action = std::env::args().nth(1);
    if matches!(action.as_deref(), Some("--register-steamvr") | Some("--unregister-steamvr")) {
      let result = if action.as_deref() == Some("--register-steamvr") {
        qptp_lib::steamvr_install::register()
      } else {
        qptp_lib::steamvr_install::unregister()
      };
      if let Err(error) = result {
        eprintln!("SteamVR registration failed: {error}");
        std::process::exit(1);
      }
      return;
    }
    use windows_sys::Win32::{Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS}, System::Threading::CreateMutexW};
    let name: Vec<u16> = "Local\\QPTPAppSingleInstance\0".encode_utf16().collect();
    let instance = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if instance.is_null() || unsafe { GetLastError() } == ERROR_ALREADY_EXISTS { return; }
    qptp_lib::run();
    unsafe { CloseHandle(instance); }
    return;
  }
  #[cfg(not(windows))]
  qptp_lib::run();
}

