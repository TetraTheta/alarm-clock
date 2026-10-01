#![cfg_attr(not(windows), allow(dead_code, unused_imports))]
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

#[cfg(not(windows))]
compile_error!("AlarmClock supports Windows only.");

mod configuration;
mod notification;
mod single_instance;
mod tray;

use std::process::ExitCode;

use configuration::load_or_create;
use notification::Notifier;
use single_instance::SingleInstance;
use tray::run_tray;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};

fn main() -> ExitCode {
  let _instance = match SingleInstance::acquire() {
    Ok(Some(instance)) => instance,
    Ok(None) => return ExitCode::SUCCESS,
    Err(error) => return fail(&error),
  };

  let config = match load_or_create() {
    Ok(config) => config,
    Err(error) => return fail(&error),
  };
  let _apartment = match ComApartment::initialize() {
    Ok(apartment) => apartment,
    Err(error) => return fail(&error),
  };
  let notifier = match Notifier::new(&config.notification, &config.sound, &config.path) {
    Ok(notifier) => notifier,
    Err(error) => return fail(&error),
  };

  match run_tray(config.alarms, notifier, _instance.shutdown_event()) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => fail(&error),
  }
}

struct ComApartment;

impl ComApartment {
  fn initialize() -> Result<Self, String> {
    let result = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    if result < 0 {
      return Err(format!("Cannot initialize Windows Runtime (HRESULT 0x{:08X}).", result as u32));
    }
    Ok(Self)
  }
}

impl Drop for ComApartment {
  fn drop(&mut self) {
    unsafe { CoUninitialize() };
  }
}

fn fail(message: &str) -> ExitCode {
  let title = wide("Alarm Clock");
  let message = wide(message);
  unsafe {
    MessageBoxW(HWND::default(), message.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
  }
  ExitCode::FAILURE
}

fn wide(value: &str) -> Vec<u16> {
  value.encode_utf16().chain(Some(0)).collect()
}
