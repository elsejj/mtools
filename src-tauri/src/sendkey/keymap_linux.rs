use std::collections::HashMap;

use std::sync::OnceLock;

fn linux_key_map() -> &'static HashMap<&'static str, &'static str> {
  static LINUX_KEY_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

  LINUX_KEY_MAP.get_or_init(|| {
    let mut map = HashMap::new();
    // Modifier keys
    map.insert("ctrl", "29");
    map.insert("shift", "42");
    map.insert("alt", "56");
    map.insert("meta", "125"); // windows key / command key

    // common keys
    map.insert("a", "30");
    map.insert("b", "48");
    map.insert("c", "46");
    map.insert("d", "32");
    map.insert("e", "18");
    map.insert("f", "33");
    map.insert("g", "34");
    map.insert("h", "35");
    map.insert("i", "23");
    map.insert("j", "36");
    map.insert("k", "37");
    map.insert("l", "38");
    map.insert("m", "50");
    map.insert("n", "49");
    map.insert("o", "24");
    map.insert("p", "25");
    map.insert("q", "16");
    map.insert("r", "19");
    map.insert("s", "31");
    map.insert("t", "20");
    map.insert("u", "22");
    map.insert("v", "47");
    map.insert("w", "17");
    map.insert("x", "45");
    map.insert("y", "21");
    map.insert("z", "44");

    // number keys
    map.insert("0", "11");
    map.insert("1", "2");
    map.insert("2", "3");
    map.insert("3", "4");
    map.insert("4", "5");
    map.insert("5", "6");
    map.insert("6", "7");
    map.insert("7", "8");
    map.insert("8", "9");
    map.insert("9", "10");

    // special keys
    map.insert("enter", "28");
    map.insert("space", "57");
    map.insert("backspace", "14");
    map.insert("tab", "15");
    map.insert("esc", "1");
    map.insert("insert", "110");
    map.insert("delete", "111");
    map.insert("home", "102");
    map.insert("end", "107");
    map.insert("pageup", "104");
    map.insert("pagedown", "109");
    map.insert("left", "105");
    map.insert("up", "103");
    map.insert("right", "106");
    map.insert("down", "108");
    map.insert("f1", "59");
    map.insert("f2", "60");
    map.insert("f3", "61");
    map.insert("f4", "62");
    map.insert("f5", "63");
    map.insert("f6", "64");
    map.insert("f7", "65");
    map.insert("f8", "66");
    map.insert("f9", "67");
    map.insert("f10", "68");
    map.insert("f11", "87");
    map.insert("f12", "88");

    map
  })
}

pub(crate) fn build_ydotool_key_sequence(keys: &str) -> Vec<String> {
  let key_map = linux_key_map();
  let mut sequence = Vec::new();

  let codes = keys
    .to_ascii_lowercase()
    .split(|c| c == ' ' || c == '+' || c == '-')
    .map_while(|k| key_map.get(k))
    .collect::<Vec<_>>();

  if codes.is_empty() {
    return sequence;
  }

  sequence.push("key".to_string());

  codes.iter().for_each(|&&code| {
    sequence.push(format!("{}:1", code)); // key down
  });
  codes.iter().rev().for_each(|&&code| {
    sequence.push(format!("{}:0", code)); // key up
  });

  sequence
}
