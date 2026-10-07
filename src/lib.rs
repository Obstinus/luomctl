pub mod cipher;
pub mod device;

/// Button number printed on the mouse picture (1..=10) -> slot in the table.
/// From the index switch in `apply_settings` (0x411BF0).
pub const SLOT: [usize; 10] = [0, 1, 2, 8, 9, 5, 6, 3, 4, 7];

pub const NAMED: &[(&str, [u8; 4])] = &[
    ("left", [0x01, 0x00, 0xf0, 0x00]),
    ("right", [0x01, 0x00, 0xf1, 0x00]),
    ("middle", [0x01, 0x00, 0xf2, 0x00]),
    ("back", [0x01, 0x00, 0xf3, 0x00]),
    ("forward", [0x01, 0x00, 0xf4, 0x00]),
    ("wheel-up", [0x01, 0x00, 0xf7, 0x00]),
    ("wheel-down", [0x01, 0x00, 0xf8, 0x00]),
    ("dpi-loop", [0x07, 0x00, 0x03, 0x00]),
    ("rgb-toggle", [0x08, 0x00, 0x03, 0x00]),
    ("off", [0x00, 0x00, 0x00, 0x00]),
];

fn hex_byte(s: &str) -> Option<u8> {
    u8::from_str_radix(s, 16).ok()
}

pub fn parse_action(s: &str) -> Option<[u8; 4]> {
    if let Some((_, v)) = NAMED.iter().find(|(n, _)| *n == s) {
        return Some(*v);
    }
    let parts: Vec<&str> = s.split(':').collect();
    match parts.as_slice() {
        ["key", k] => Some([0x00, 0x00, hex_byte(k)?, 0x00]),
        ["combo", m, k] => Some([0x00, hex_byte(m)?, hex_byte(k)?, 0x00]),
        ["raw", h] if h.len() == 8 => {
            let mut v = [0u8; 4];
            for (i, b) in v.iter_mut().enumerate() {
                *b = hex_byte(&h[i * 2..i * 2 + 2])?;
            }
            Some(v)
        }
        _ => None,
    }
}

pub fn describe(v: [u8; 4]) -> String {
    if let Some((n, _)) = NAMED.iter().find(|(_, x)| *x == v) {
        return n.to_string();
    }
    match v {
        [0x00, 0x00, k, 0x00] => format!("key:{k:02x}"),
        [0x00, m, k, 0x00] => format!("combo:{m:02x}:{k:02x}"),
        [0x0a, ..] => format!("fire-key (raw:{})", hexs(v)),
        [0x09, ..] => format!("macro {} (raw:{})", v[2].wrapping_sub(1), hexs(v)),
        _ => format!("raw:{}", hexs(v)),
    }
}

fn hexs(v: [u8; 4]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

