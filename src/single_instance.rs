use std::ptr;

use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{CreateEventW, CreateMutexW, SetEvent, WaitForSingleObject};

const MUTEX_NAME: &str = "Local\\AlarmClock.01A1C420-7FE0-4A87-9DA4-529C93E9B821\0";
const SHUTDOWN_EVENT_NAME: &str = "Local\\AlarmClock.Shutdown.01A1C420-7FE0-4A87-9DA4-529C93E9B821\0";

pub(crate) struct SingleInstance {
  mutex: HANDLE,
  shutdown_event: HANDLE,
}

impl SingleInstance {
  pub(crate) fn acquire() -> Result<Option<Self>, String> {
    let mutex_name = MUTEX_NAME.encode_utf16().collect::<Vec<_>>();
    let mutex = unsafe { CreateMutexW(ptr::null(), 1, mutex_name.as_ptr()) };
    if mutex.is_null() {
      return Err(format!("Cannot create the single-instance mutex (error {}).", unsafe { GetLastError() }));
    }
    let already_running = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;

    let event_name = SHUTDOWN_EVENT_NAME.encode_utf16().collect::<Vec<_>>();
    let shutdown_event = unsafe { CreateEventW(ptr::null(), 0, 0, event_name.as_ptr()) };
    if shutdown_event.is_null() {
      unsafe { CloseHandle(mutex) };
      return Err(format!("Cannot create the shutdown event (error {}).", unsafe { GetLastError() }));
    }

    if already_running {
      unsafe { SetEvent(shutdown_event) };
      match unsafe { WaitForSingleObject(mutex, 5_000) } {
        WAIT_OBJECT_0 | WAIT_ABANDONED => {},
        WAIT_TIMEOUT => {
          unsafe {
            CloseHandle(shutdown_event);
            CloseHandle(mutex);
          }
          return Err("The existing Alarm Clock instance did not exit within five seconds.".to_owned());
        },
        _ => {
          let error = unsafe { GetLastError() };
          unsafe {
            CloseHandle(shutdown_event);
            CloseHandle(mutex);
          }
          return Err(format!("Cannot wait for the existing Alarm Clock instance (error {error})."));
        },
      }
    }
    Ok(Some(Self { mutex, shutdown_event }))
  }

  pub(crate) fn shutdown_event(&self) -> HANDLE {
    self.shutdown_event
  }
}

impl Drop for SingleInstance {
  fn drop(&mut self) {
    unsafe {
      CloseHandle(self.shutdown_event);
      CloseHandle(self.mutex);
    }
  }
}
