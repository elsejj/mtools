#[cfg(target_os = "windows")]
pub mod sendkey_win;

#[cfg(target_os = "linux")]
mod sendkey_linux;

#[cfg(target_os = "linux")]
mod keymap_linux;

#[cfg(target_os = "macos")]
pub mod sendkey_mac;

pub fn send_keys<S: AsRef<str>>(keys: S) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  return sendkey_win::send_keys(keys.as_ref());

  #[cfg(target_os = "linux")]
  return sendkey_linux::send_keys_native(keys.as_ref());

  #[cfg(target_os = "macos")]
  return sendkey_mac::send_keys(keys.as_ref());

  #[allow(unreachable_code)]
  Err("send_keys is not supported on this platform".into())
}

#[cfg(target_os = "linux")]
pub static COPY_KEY: &'static str = "Ctrl+Insert";
#[cfg(target_os = "macos")]
pub static COPY_KEY: &'static str = "Cmd+C";
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub static COPY_KEY: &'static str = "Ctrl+C";

#[cfg(target_os = "linux")]
pub static PASTE_KEY: &'static str = "Shift+Insert";
#[cfg(target_os = "macos")]
pub static PASTE_KEY: &'static str = "Cmd+V";
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub static PASTE_KEY: &'static str = "Ctrl+V";
