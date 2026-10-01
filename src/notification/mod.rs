use std::env;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::ptr;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};
use windows_sys::Win32::Graphics::Gdi::{
  BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, HGDIOBJ, SelectObject,
};
use windows_sys::Win32::UI::Shell::ExtractIconExW;
use windows_sys::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx, GetSystemMetrics, SM_CXICON, SM_CYICON};
use winrt_toast_reborn::content::audio::{LoopingSound, Sound};
use winrt_toast_reborn::content::image::ImagePlacement;
use winrt_toast_reborn::{Audio, Image, Toast, ToastDuration, ToastManager, register};

use crate::configuration::{NotificationConfig, SoundConfig};

const AUM_ID: &str = "AlarmClock.Desktop";

pub(crate) struct Notifier {
  audio: Option<AudioPlayer>,
  config: NotificationConfig,
  icon: Option<PathBuf>,
  manager: ToastManager,
  toast_sound: Option<Sound>,
}

struct AudioPlayer {
  path: PathBuf,
  player: Player,
  _stream: MixerDeviceSink,
}

impl Notifier {
  pub(crate) fn new(config: &NotificationConfig, sound_config: &SoundConfig, config_path: &Path) -> Result<Self, String> {
    let icon = prepare_icon(&config.icon, config_path).ok();
    register(AUM_ID, "Alarm Clock", icon.as_deref()).map_err(|error| format!("Cannot register notifications: {error}"))?;
    let sound = configured_sound(&sound_config.sound, config_path);
    let (audio, toast_sound) = match sound {
      ConfiguredSound::Default => (None, None),
      ConfiguredSound::System(sound) => (None, Some(sound)),
      ConfiguredSound::File(path) => (Some(AudioPlayer::new(path, sound_config.volume)?), Some(Sound::None)),
    };
    Ok(Self { audio, config: config.clone(), icon, manager: ToastManager::new(AUM_ID), toast_sound })
  }

  pub(crate) fn show(&self) {
    let mut toast = Toast::new();
    toast.text1(&self.config.title).text2(&self.config.desc).duration(ToastDuration::Long);
    if let Some(path) = &self.icon
      && let Ok(image) = Image::new_local(path)
    {
      toast.image(1, image.with_placement(ImagePlacement::AppLogoOverride).with_alt("Alarm Clock"));
    }
    if let Some(sound) = &self.toast_sound {
      toast.audio(Audio::new(sound.clone()));
    }
    let _ = self.manager.show(&toast);
    if let Some(audio) = &self.audio {
      audio.play();
    }
  }
}

impl AudioPlayer {
  fn new(path: PathBuf, volume: f32) -> Result<Self, String> {
    let file = fs::File::open(&path).map_err(|error| format!("Cannot open sound file '{}': {error}", path.display()))?;
    Decoder::try_from(file).map_err(|error| format!("Cannot decode sound file '{}': {error}", path.display()))?;
    let stream = DeviceSinkBuilder::open_default_sink().map_err(|error| format!("Cannot open the default audio device: {error}"))?;
    let player = Player::connect_new(stream.mixer());
    player.set_volume(volume / 100.0);
    Ok(Self { path, player, _stream: stream })
  }

  fn play(&self) {
    let Ok(file) = fs::File::open(&self.path) else { return };
    let Ok(source) = Decoder::try_from(file) else { return };
    self.player.clear();
    self.player.append(source);
    self.player.play();
  }
}

enum ConfiguredSound {
  Default,
  File(PathBuf),
  System(Sound),
}

fn configured_sound(value: &str, config_path: &Path) -> ConfiguredSound {
  match value.trim().to_ascii_lowercase().as_str() {
    "" | "default" => ConfiguredSound::Default,
    "im" => ConfiguredSound::System(Sound::IM),
    "mail" => ConfiguredSound::System(Sound::Mail),
    "reminder" => ConfiguredSound::System(Sound::Reminder),
    "sms" => ConfiguredSound::System(Sound::SMS),
    "alarm" | "alarm1" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm)),
    "alarm2" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm2)),
    "alarm3" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm3)),
    "alarm4" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm4)),
    "alarm5" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm5)),
    "alarm6" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm6)),
    "alarm7" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm7)),
    "alarm8" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm8)),
    "alarm9" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm9)),
    "alarm10" => ConfiguredSound::System(Sound::Looping(LoopingSound::Alarm10)),
    _ => {
      let path = PathBuf::from(value.trim());
      ConfiguredSound::File(if path.is_absolute() { path } else { config_path.parent().unwrap_or_else(|| Path::new("")).join(path) })
    },
  }
}

fn prepare_icon(value: &str, config_path: &Path) -> Result<PathBuf, String> {
  let (raw_path, index) = parse_icon_spec(value);
  let executable = env::current_exe().map_err(|error| format!("Cannot locate the executable icon: {error}"))?;
  let source = resolve_icon_source(raw_path, config_path, &executable);
  let (rgba, width, height) = extract_icon(&source, index)?;
  let directory = env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(env::temp_dir).join("AlarmClock");
  fs::create_dir_all(&directory).map_err(|error| format!("Cannot create '{}': {error}", directory.display()))?;
  let destination = directory.join("notification-icon.png");
  let file = fs::File::create(&destination).map_err(|error| format!("Cannot create '{}': {error}", destination.display()))?;
  let mut encoder = png::Encoder::new(file, width, height);
  encoder.set_color(png::ColorType::Rgba);
  encoder.set_depth(png::BitDepth::Eight);
  encoder
    .write_header()
    .and_then(|mut writer| writer.write_image_data(&rgba))
    .map_err(|error| format!("Cannot encode notification icon: {error}"))?;
  Ok(destination)
}

fn resolve_icon_source(value: &str, config_path: &Path, executable_path: &Path) -> PathBuf {
  if value.is_empty() {
    return executable_path.to_path_buf();
  }
  let path = PathBuf::from(value);
  if path.is_absolute() {
    path
  } else if path.components().count() == 1
    && path
      .extension()
      .and_then(|extension| extension.to_str())
      .is_some_and(|extension| extension.eq_ignore_ascii_case("dll") || extension.eq_ignore_ascii_case("exe"))
  {
    env::var_os("WINDIR").map(PathBuf::from).unwrap_or_default().join("System32").join(path)
  } else {
    config_path.parent().unwrap_or_else(|| Path::new("")).join(path)
  }
}

fn parse_icon_spec(value: &str) -> (&str, i32) {
  let value = value.trim();
  match value.rsplit_once(':') {
    Some((path, index)) if index.parse::<i32>().is_ok() => (path, index.parse().unwrap_or_default()),
    _ => (value, 0),
  }
}

fn extract_icon(path: &Path, index: i32) -> Result<(Vec<u8>, u32, u32), String> {
  let path = path.as_os_str().to_string_lossy().encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let mut icon = ptr::null_mut();
  if unsafe { ExtractIconExW(path.as_ptr(), index, &mut icon, ptr::null_mut(), 1) } == 0 || icon.is_null() {
    return Err("Cannot extract the configured notification icon.".to_owned());
  }

  let width = unsafe { GetSystemMetrics(SM_CXICON) }.max(1) as u32;
  let height = unsafe { GetSystemMetrics(SM_CYICON) }.max(1) as u32;
  let mut pixels = ptr::null_mut();
  let info = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width as i32,
      biHeight: -(height as i32),
      biPlanes: 1,
      biBitCount: 32,
      biCompression: BI_RGB,
      ..unsafe { mem::zeroed() }
    },
    ..unsafe { mem::zeroed() }
  };
  let dc = unsafe { CreateCompatibleDC(ptr::null_mut()) };
  let bitmap = unsafe { CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, ptr::null_mut(), 0) };
  if dc.is_null() || bitmap.is_null() || pixels.is_null() {
    unsafe {
      if !bitmap.is_null() {
        DeleteObject(bitmap as HGDIOBJ);
      }
      if !dc.is_null() {
        DeleteDC(dc);
      }
      DestroyIcon(icon);
    }
    return Err("Cannot create the notification icon bitmap.".to_owned());
  }
  let old = unsafe { SelectObject(dc, bitmap as HGDIOBJ) };
  let drawn = unsafe { DrawIconEx(dc, 0, 0, icon, width as i32, height as i32, 0, ptr::null_mut(), DI_NORMAL) };
  let mut rgba = unsafe { std::slice::from_raw_parts(pixels.cast::<u8>(), (width * height * 4) as usize).to_vec() };
  for pixel in rgba.as_chunks_mut::<4>().0 {
    pixel.swap(0, 2);
  }
  unsafe {
    SelectObject(dc, old);
    DeleteObject(bitmap as HGDIOBJ);
    DeleteDC(dc);
    DestroyIcon(icon);
  }
  if drawn == 0 {
    return Err("Cannot render the configured notification icon.".to_owned());
  }
  Ok((rgba, width, height))
}

#[cfg(test)]
mod tests;
