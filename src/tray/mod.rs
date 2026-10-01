use std::mem;
use std::ptr;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use windows_sys::Win32::Foundation::{HANDLE, SYSTEMTIME, WAIT_OBJECT_0};
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
use windows_sys::Win32::System::SystemInformation::GetLocalTime;
use windows_sys::Win32::System::Threading::WaitForSingleObject;
use windows_sys::Win32::UI::WindowsAndMessaging::{
  DispatchMessageW, GetMessageW, KillTimer, MSG, PostQuitMessage, SetTimer, TranslateMessage, WM_TIMER,
};

use crate::configuration::Alarm;
use crate::notification::Notifier;

const EXIT_ID: &str = "exit";

struct AlarmItem {
  alarm: Alarm,
  item: CheckMenuItem,
  last_date: u32,
}

pub(crate) fn run_tray(alarms: Vec<Alarm>, notifier: Notifier, shutdown_event: HANDLE) -> Result<(), String> {
  let menu = Menu::new();
  let mut items = alarms
    .into_iter()
    .map(|alarm| {
      let item = CheckMenuItem::with_id(format!("alarm-{}", alarm.second_of_day), &alarm.label, true, true, None);
      menu.append(&item).map_err(|error| error.to_string())?;
      Ok(AlarmItem { alarm, item, last_date: 0 })
    })
    .collect::<Result<Vec<_>, String>>()?;
  menu.append(&PredefinedMenuItem::separator()).map_err(|error| error.to_string())?;
  let exit = MenuItem::with_id(EXIT_ID, exit_label(), true, None);
  menu.append(&exit).map_err(|error| error.to_string())?;

  let icon = Icon::from_resource(1, Some((32, 32))).map_err(|error| format!("Cannot load the tray icon: {error}"))?;
  let _tray = TrayIconBuilder::new()
    .with_tooltip("Alarm Clock")
    .with_menu(Box::new(menu))
    .with_menu_on_left_click(false)
    .with_icon(icon)
    .build()
    .map_err(|error| format!("Cannot create the tray icon: {error}"))?;

  let timer = unsafe { SetTimer(ptr::null_mut(), 0, 250, None) };
  if timer == 0 {
    return Err("Cannot start the alarm timer.".to_owned());
  }
  unsafe {
    let mut message: MSG = mem::zeroed();
    while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
      TranslateMessage(&message);
      DispatchMessageW(&message);
      if message.message == WM_TIMER {
        if WaitForSingleObject(shutdown_event, 0) == WAIT_OBJECT_0 {
          PostQuitMessage(0);
          continue;
        }
        notify_due_alarms(&mut items, &notifier);
      }
      while let Ok(event) = MenuEvent::receiver().try_recv() {
        if event.id == MenuId::new(EXIT_ID) {
          PostQuitMessage(0);
        }
      }
    }
    KillTimer(ptr::null_mut(), timer);
  }
  Ok(())
}

fn notify_due_alarms(items: &mut [AlarmItem], notifier: &Notifier) {
  let mut now: SYSTEMTIME = unsafe { mem::zeroed() };
  unsafe { GetLocalTime(&mut now) };
  let date = u32::from(now.wYear) * 10_000 + u32::from(now.wMonth) * 100 + u32::from(now.wDay);
  let second_of_day = u32::from(now.wHour) * 3600 + u32::from(now.wMinute) * 60 + u32::from(now.wSecond);
  for item in items {
    if is_due(item.alarm.second_of_day, item.last_date, date, second_of_day) && item.item.is_checked() {
      item.last_date = date;
      notifier.show();
    }
  }
}

fn is_due(alarm_second: u32, last_date: u32, current_date: u32, current_second: u32) -> bool {
  alarm_second == current_second && last_date != current_date
}

fn exit_label() -> &'static str {
  let language = unsafe { GetUserDefaultUILanguage() };
  if language & 0x03ff == 0x12 { "종료" } else { "Exit" }
}

#[cfg(test)]
mod tests;
