use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use configparser::ini::Ini;

const CONFIG_FILE_NAME: &str = "AlarmClock.ini";
const DEFAULT_CONFIG: &str = "; Times use [AM|PM] <hour>:<minute>[:<second>]. Without AM/PM, both times are used.\r\n\
[alarm]\r\n\
1=AM 10:00:30\r\n\
2=PM 12:00\r\n\
3=1:15\r\n\
\r\n\
[notification]\r\n\
title=Time's Up!\r\n\
desc=Check your belongings before leaving\r\n\
; Leave empty to use the icon embedded in AlarmClock.exe.\r\n\
icon=\r\n\
\r\n\
[sound]\r\n\
; Default, IM, Mail, Reminder, SMS, Alarm, Alarm2 ... Alarm10\r\n\
sound=Default\r\n\
; 0 through 100. Volume applies to sound files; Windows controls Toast sound volume.\r\n\
volume=100\r\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Alarm {
  pub(crate) label: String,
  pub(crate) second_of_day: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NotificationConfig {
  pub(crate) desc: String,
  pub(crate) icon: String,
  pub(crate) title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SoundConfig {
  pub(crate) sound: String,
  pub(crate) volume: f32,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Config {
  pub(crate) alarms: Vec<Alarm>,
  pub(crate) notification: NotificationConfig,
  pub(crate) path: PathBuf,
  pub(crate) sound: SoundConfig,
}

pub(crate) fn load_or_create() -> Result<Config, String> {
  let directory = env::current_exe()
    .ok()
    .and_then(|path| path.parent().map(Path::to_path_buf))
    .ok_or_else(|| "Cannot determine the executable directory.".to_owned())?;
  let path = directory.join(CONFIG_FILE_NAME);
  if !path.exists() {
    fs::write(&path, DEFAULT_CONFIG).map_err(|error| format!("Cannot create '{}': {error}", path.display()))?;
  }
  load(&path)
}

fn load(path: &Path) -> Result<Config, String> {
  let mut ini = Ini::new();
  ini.load(path.to_string_lossy().as_ref()).map_err(|error| format!("Cannot read '{}': {error}", path.display()))?;

  let alarms = ini
    .get_map_ref()
    .get("alarm")
    .into_iter()
    .flat_map(|section| section.values())
    .filter_map(|value| value.as_deref())
    .flat_map(parse_alarm)
    .collect::<BTreeSet<_>>()
    .into_iter()
    .map(|second_of_day| Alarm { label: format_alarm(second_of_day), second_of_day })
    .collect();

  let notification = NotificationConfig {
    desc: ini.get("notification", "desc").unwrap_or_default(),
    icon: ini.get("notification", "icon").unwrap_or_default(),
    title: ini.get("notification", "title").unwrap_or_else(|| "Time's Up!".to_owned()),
  };
  let volume =
    ini.get("sound", "volume").and_then(|value| value.trim().parse::<f32>().ok()).filter(|value| (0.0..=100.0).contains(value)).unwrap_or(100.0);
  let sound = SoundConfig { sound: ini.get("sound", "sound").unwrap_or_else(|| "Default".to_owned()), volume };
  Ok(Config { alarms, notification, path: path.to_path_buf(), sound })
}

fn parse_alarm(value: &str) -> Vec<u32> {
  let parts = value.split_whitespace().collect::<Vec<_>>();
  let (period, time) = match parts.as_slice() {
    [time] => (None, *time),
    [period, time] if period.eq_ignore_ascii_case("AM") || period.eq_ignore_ascii_case("PM") => (Some(*period), *time),
    _ => return Vec::new(),
  };
  let components = time.split(':').collect::<Vec<_>>();
  if !(2..=3).contains(&components.len()) {
    return Vec::new();
  }
  let Ok(hour) = components[0].parse::<u32>() else { return Vec::new() };
  let Ok(minute) = components[1].parse::<u32>() else { return Vec::new() };
  let Ok(second) = components.get(2).copied().unwrap_or("0").parse::<u32>() else { return Vec::new() };
  if !(1..=12).contains(&hour) || minute > 59 || second > 59 {
    return Vec::new();
  }

  let base = (hour % 12) * 3600 + minute * 60 + second;
  match period.map(str::to_ascii_uppercase).as_deref() {
    Some("AM") => vec![base],
    Some("PM") => vec![base + 12 * 3600],
    None => vec![base, base + 12 * 3600],
    _ => Vec::new(),
  }
}

fn format_alarm(second_of_day: u32) -> String {
  let hour = second_of_day / 3600;
  let minute = second_of_day % 3600 / 60;
  let second = second_of_day % 60;
  let period = if hour < 12 { "AM" } else { "PM" };
  let hour12 = match hour % 12 {
    0 => 12,
    value => value,
  };
  format!("{period} {hour12}:{minute:02}:{second:02}")
}

#[cfg(test)]
mod tests;
