use core_graphics::event::{CGEvent, CGEventTapLocation, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Map a key token to macOS Carbon Virtual Keycode
fn mac_key_code(key: &str) -> Option<CGKeyCode> {
  match key {
    // Modifiers
    "CMD" | "COMMAND" | "META" | "WIN" => Some(0x37), // kVK_Command
    "SHIFT" => Some(0x38),                            // kVK_Shift
    "ALT" | "OPTION" | "OPT" => Some(0x3A),           // kVK_Option
    "CTRL" | "CONTROL" => Some(0x3B),                 // kVK_Control

    // Letters (ANSI layout)
    "A" => Some(0x00),
    "B" => Some(0x0B),
    "C" => Some(0x08),
    "D" => Some(0x02),
    "E" => Some(0x0E),
    "F" => Some(0x03),
    "G" => Some(0x05),
    "H" => Some(0x04),
    "I" => Some(0x22),
    "J" => Some(0x26),
    "K" => Some(0x28),
    "L" => Some(0x25),
    "M" => Some(0x2E),
    "N" => Some(0x2D),
    "O" => Some(0x1F),
    "P" => Some(0x23),
    "Q" => Some(0x0C),
    "R" => Some(0x0F),
    "S" => Some(0x01),
    "T" => Some(0x11),
    "U" => Some(0x20),
    "V" => Some(0x09),
    "W" => Some(0x0D),
    "X" => Some(0x07),
    "Y" => Some(0x10),
    "Z" => Some(0x06),

    // Numbers
    "0" => Some(0x1D),
    "1" => Some(0x12),
    "2" => Some(0x13),
    "3" => Some(0x14),
    "4" => Some(0x15),
    "5" => Some(0x17),
    "6" => Some(0x16),
    "7" => Some(0x1A),
    "8" => Some(0x1C),
    "9" => Some(0x19),

    // Common Controls & Navigation
    "ENTER" | "RETURN" => Some(0x24),
    "SPACE" => Some(0x31),
    "TAB" => Some(0x30),
    "ESC" | "ESCAPE" => Some(0x35),
    "BACKSPACE" | "DELETE" => Some(0x33),
    "FORWARDDELETE" => Some(0x75),
    "LEFT" => Some(0x7B),
    "RIGHT" => Some(0x7C),
    "DOWN" => Some(0x7D),
    "UP" => Some(0x7E),
    "HOME" => Some(0x73),
    "END" => Some(0x77),
    "PAGEUP" => Some(0x74),
    "PAGEDOWN" => Some(0x79),

    // Function keys
    "F1" => Some(0x7A),
    "F2" => Some(0x78),
    "F3" => Some(0x63),
    "F4" => Some(0x76),
    "F5" => Some(0x60),
    "F6" => Some(0x61),
    "F7" => Some(0x62),
    "F8" => Some(0x64),
    "F9" => Some(0x65),
    "F10" => Some(0x6D),
    "F11" => Some(0x67),
    "F12" => Some(0x6F),

    _ => None,
  }
}

pub(crate) fn send_keys(keys: &str) -> Result<(), String> {
  let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
    .map_err(|_| "Failed to create CGEventSource".to_string())?;

  let key_codes: Vec<CGKeyCode> = keys
    .to_ascii_uppercase()
    .split(|c| c == ' ' || c == '+' || c == '-')
    .filter(|s| !s.is_empty())
    .map(|s| mac_key_code(s).ok_or_else(|| format!("Unsupported key for macOS: {}", s)))
    .collect::<Result<Vec<_>, String>>()?;

  if key_codes.is_empty() {
    return Err("No keys to send".to_string());
  }

  // Key Down in forward order
  for &kc in &key_codes {
    let event = CGEvent::new_keyboard_event(source.clone(), kc, true)
      .map_err(|_| format!("Failed to create key down event for keycode {}", kc))?;
    event.post(CGEventTapLocation::HID);
  }

  // Key Up in reverse order
  for &kc in key_codes.iter().rev() {
    let event = CGEvent::new_keyboard_event(source.clone(), kc, false)
      .map_err(|_| format!("Failed to create key up event for keycode {}", kc))?;
    event.post(CGEventTapLocation::HID);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_mac_key_codes() {
    assert_eq!(mac_key_code("CMD"), Some(0x37));
    assert_eq!(mac_key_code("COMMAND"), Some(0x37));
    assert_eq!(mac_key_code("META"), Some(0x37));
    assert_eq!(mac_key_code("SHIFT"), Some(0x38));
    assert_eq!(mac_key_code("ALT"), Some(0x3A));
    assert_eq!(mac_key_code("OPTION"), Some(0x3A));
    assert_eq!(mac_key_code("CTRL"), Some(0x3B));
    assert_eq!(mac_key_code("C"), Some(0x08));
    assert_eq!(mac_key_code("V"), Some(0x09));
    assert_eq!(mac_key_code("ENTER"), Some(0x24));
    assert_eq!(mac_key_code("SPACE"), Some(0x31));
  }
}
