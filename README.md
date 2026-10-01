# Alarm Clock

Alarm Clock is a lightweight Windows 10/11 tray application that sends daily reminders through Windows Notification Center.

## Features

- Daily alarms with optional seconds
- Windows Notification Center notifications
- A tray menu for temporarily enabling or disabling individual alarm times
- The Windows default notification sound, built-in Toast sounds, or local audio files
- A configurable notification icon, including icons stored in DLL or EXE resources
- Automatic replacement of an already running instance

## Quick start

1. Place `AlarmClock.exe` in any writable directory.
2. Run it once. If `AlarmClock.ini` does not exist beside the executable, Alarm Clock creates a default configuration file.
3. Edit `AlarmClock.ini` and run `AlarmClock.exe` again. The new instance asks the existing instance to exit, then starts with the updated configuration.
4. Right-click the tray icon to temporarily enable or disable alarms. These check states reset when the application restarts.
5. Select the localized Exit command to close the application.

Configuration changes are read only at startup.

## Configuration

```ini
[alarm]
leaving=AM 10:00:30
lunch=PM 12:00
check=1:15

[notification]
title=Time's Up!
desc=Check your belongings before leaving
icon=

[sound]
sound=Default
volume=100
```

The file must be named `AlarmClock.ini` and stored beside `AlarmClock.exe`.

### Alarms

Each entry in `[alarm]` has an arbitrary, unique key and a time value:

```text
[AM|PM] <hour>:<minute>[:<second>]
```

Examples:

```ini
[alarm]
work=AM 10:00:30
noon=PM 12:00
check=1:15
```

- Hours use the 12-hour clock and must be between `1` and `12`.
- Minutes and seconds must be between `0` and `59`.
- Seconds default to `00` when omitted.
- A value without `AM` or `PM` creates both times. For example, `1:15` creates `AM 1:15:00` and `PM 1:15:00`.
- Duplicate times are combined into one tray-menu entry.
- Invalid alarm entries are ignored without preventing other alarms from running.
- Every enabled alarm repeats daily.

### Notification

The `[notification]` section supports these keys:

| Key | Description |
| --- | --- |
| `title` | Notification title. Defaults to `Time's Up!`. |
| `desc` | Notification body text. |
| `icon` | Optional image, icon, DLL, or EXE path, followed by an optional icon index. |

If `icon` is missing or empty, Alarm Clock extracts and uses the icon embedded in `AlarmClock.exe`. That icon comes from `resource/main.ico` when the application is built.

Icon examples:

```ini
[notification]
icon=
; Alternatively, use one of these:
; icon=shell32.dll:10
; icon=C:\Windows\System32\imageres.dll:15
; icon=icons\alarm.ico
```

Relative file paths are resolved from the directory containing `AlarmClock.ini`. A bare DLL or EXE name, such as `shell32.dll`, is resolved from the Windows system directory. The selected icon is converted to PNG and cached under `%LOCALAPPDATA%\AlarmClock` for use by Notification Center.

### Sound

`sound` may be empty, a supported Windows Toast sound name, or an audio file path:

```ini
[sound]
sound=Default
volume=100
```

Supported Toast sound names are:

- `Default`
- `IM`
- `Mail`
- `Reminder`
- `SMS`
- `Alarm` or `Alarm1` through `Alarm10`

An empty value behaves like `Default`. Windows controls the volume of Toast sounds through its system notification settings, so `volume` does not override named Toast sounds.

For an audio file, use an absolute path or a path relative to `AlarmClock.ini`:

```ini
[sound]
sound=sounds\alarm.mp3
volume=35
```

- `volume` accepts values from `0` through `100` and defaults to `100`.
- Volume is applied to audio files.
- Supported file formats are WAV, MP3, MP4/M4A, FLAC, and OGG Vorbis.
- File playback stops when Alarm Clock exits or is replaced by a new instance.
- A missing, unreadable, or unsupported audio file prevents startup and displays an error.

## Tray menu

Alarm times are sorted chronologically. Each checked entry is active; clear its check mark to disable that time until the application restarts. Permanent changes must be made in `AlarmClock.ini`.

The final menu item uses the Korean translation of Exit when the Windows UI language is Korean and `Exit` otherwise.

## Build from source

Install a current Rust toolchain with the MSVC Windows target, then run:

```powershell
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --release
```

The executable is written to `target\release\AlarmClock.exe`.

## Troubleshooting

- Confirm that notifications are enabled for Alarm Clock in Windows notification settings.
- Run the application again after editing `AlarmClock.ini`; the running instance does not watch the file for changes.
- If no tray icon appears, check the taskbar overflow area.
- If a custom audio file fails, verify its path, format, and permissions.
