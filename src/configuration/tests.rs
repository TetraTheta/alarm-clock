use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn parses_sorts_expands_and_skips_invalid_alarms() {
  let directory = temporary_directory();
  let path = directory.join("AlarmClock.ini");
  fs::write(
    &path,
    "[alarm]\nfirst=AM 10:00:30\nsecond=PM 12:00\nthird=1:15\nduplicate=AM 1:15\nbad=25:00\n[notification]\ntitle=Done\n[sound]\nsound=alarm.mp3\nvolume=35\n",
  )
  .unwrap();

  let config = load(&path).unwrap();
  assert_eq!(config.alarms.iter().map(|alarm| alarm.label.as_str()).collect::<Vec<_>>(), ["AM 1:15:00", "AM 10:00:30", "PM 12:00:00", "PM 1:15:00"]);
  assert_eq!(config.notification.title, "Done");
  assert_eq!(config.sound.sound, "alarm.mp3");
  assert_eq!(config.sound.volume, 35.0);

  fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rejects_malformed_time_components() {
  assert!(parse_alarm("AM 0:00").is_empty());
  assert!(parse_alarm("PM 12:60").is_empty());
  assert!(parse_alarm("AM 1:02:60").is_empty());
  assert!(parse_alarm("AM 1").is_empty());
  assert!(parse_alarm("noon 1:00").is_empty());
}

#[test]
fn defaults_invalid_volume_to_one_hundred() {
  let directory = temporary_directory();
  let path = directory.join("AlarmClock.ini");
  fs::write(&path, "[sound]\nvolume=101\n").unwrap();

  assert_eq!(load(&path).unwrap().sound.volume, 100.0);

  fs::remove_dir_all(directory).unwrap();
}

fn temporary_directory() -> PathBuf {
  let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
  let path = env::temp_dir().join(format!("alarm-clock-test-{}-{unique}", std::process::id()));
  fs::create_dir(&path).unwrap();
  path
}
