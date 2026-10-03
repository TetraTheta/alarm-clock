use super::*;

#[test]
fn parses_icon_path_without_confusing_drive_letter() {
  assert_eq!(parse_icon_spec(r"C:\Windows\System32\shell32.dll:10"), (r"C:\Windows\System32\shell32.dll", 10));
  assert_eq!(parse_icon_spec(r"C:\icons\alarm.ico"), (r"C:\icons\alarm.ico", 0));
}

#[test]
fn uses_executable_icon_when_notification_icon_is_empty() {
  let executable = Path::new(r"C:\AlarmClock\AlarmClock.exe");
  let config = Path::new(r"C:\AlarmClock\AlarmClock.ini");

  assert_eq!(resolve_icon_source("", config, executable), executable);
  assert_eq!(resolve_icon_source("alarm.ico", config, executable), PathBuf::from(r"C:\AlarmClock\alarm.ico"));
}

#[test]
fn treats_unknown_sound_name_as_a_path() {
  let config = Path::new(r"C:\AlarmClock\AlarmClock.ini");
  match configured_sound("sounds/alarm.mp3", config) {
    ConfiguredSound::File(path) => assert_eq!(path, PathBuf::from(r"C:\AlarmClock\sounds/alarm.mp3")),
    _ => panic!("expected a file sound"),
  }
}

#[test]
fn extracts_the_notification_center_icon_size() {
  let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resource/main.ico");
  let (_, width, height) = extract_icon(&path, 0).expect("notification icon should be extracted");

  assert_eq!((width, height), (48, 48));
}
