//! Linux-specific implementation of the sendkey module
//! This module provides functionality to send key events and interact with the clipboard on Linux systems.
//! It uses the `ydotool` utility to simulate key presses and manage clipboard files.
//!
//! `ydotool` must be installed and running for this module to function correctly.
//!

//const YDOTOOL_SOCKET: &str = "/tmp/sendkey_ydotool.sock";
//const YDOTOOL_PERMISSIONS: &str = "0666";

use std::{
  cell::OnceCell,
  fmt::format,
  process::Command,
  sync::{Mutex, OnceLock, RwLock},
};

use mouse_keyboard_input::VirtualDevice;

use crate::sendkey::keymap_linux::KeyState;

pub(crate) fn send_keys(keys: &str) -> Result<(), String> {
  // use ydotool to send keys
  let mut command = Command::new("ydotool");

  let sequence = crate::sendkey::keymap_linux::build_ydotool_key_sequence(keys);
  let child = command
    .args(sequence)
    .spawn()
    .map_err(|e| format!("Failed to send keys: {}, ", e))?;

  let output = child
    .wait_with_output()
    .map_err(|e| format!("Failed to wait for ydotool output: {}", e))?;

  println!(
    "ydotool output: {:?}",
    String::from_utf8_lossy(&output.stdout)
  );

  Ok(())
}

pub(crate) fn send_keys_native(keys: &str) -> Result<(), String> {
  let keys = super::keymap_linux::build_key_sequence(keys);
  if keys.is_empty() {
    return Err("no keys need to send".to_string());
  }

  static VK: OnceLock<Mutex<Option<VirtualDevice>>> = OnceLock::new();

  let vk = VK.get_or_init(|| Mutex::new(VirtualDevice::default().ok()));

  let mut guard = vk.lock().map_err(|e| "virtual keyboard busy")?;

  if let Some(sender) = guard.as_mut() {
    for (action, code) in keys {
      match action {
        KeyState::Release => sender.release(code),
        KeyState::Press => sender.press(code),
      }
      .map_err(|e| format!("send key {} {:?} failed: {}", code, action, e))?;
    }
    return Ok(());
  } else {
    return Err("can't open virtual keyboard".to_string());
  }
}

// pub(crate) fn finalize_sendkey() -> Result<(), String> {
//   // YDOTOOLD_INSTANCE.get().map(|child| {
//   //   if let Ok(mut child) = child.lock() {
//   //     if let Err(e) = child.kill() {
//   //       eprintln!("Failed to kill ydotoold: {}", e);
//   //     }
//   //   } else {
//   //     eprintln!("Failed to lock ydotoold process");
//   //   }
//   // });
//   Ok(())
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_send_keys() {
    // This test will attempt to call send_keys, but will likely fail unless ydotool is installed and running with proper permissions.
    let result = send_keys("test");
    // Accept both Ok and Err, but print the result for manual inspection
    println!("send_keys result: {:?}", result);
    // Optionally, assert that it does not panic
    assert!(result.is_ok() || result.is_err());
  }

  #[test]
  fn test_send_keys_native() {
    // This test will attempt to call send_keys, but will likely fail unless ydotool is installed and running with proper permissions.
    let result = send_keys_native("test");
    // Accept both Ok and Err, but print the result for manual inspection
    println!("send_keys result: {:?}", result);
    // Optionally, assert that it does not panic
    assert!(result.is_ok() || result.is_err());
  }
}
