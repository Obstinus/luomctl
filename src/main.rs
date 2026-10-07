mod cipher;
mod device;

use device::{ButtonTable, Mouse};
use std::process::ExitCode;

/// Button number printed on the mouse picture (1..=10) -> slot in the table.
/// From the index switch in `apply_settings` (0x411BF0).
const SLOT: [usize; 10] = [0, 1, 2, 8, 9, 5, 6, 3, 4, 7];

const USAGE: &str = "\
luomctl - configure the LUOM G10 / Combaterwing mouse (04d9:a09f)

USAGE:
  luomctl show [a|b]                     show the button actions
  luomctl set <button> <action> [a|b]    change one button (default: mode a)

BUTTON: 1-10, as numbered in the official software

ACTION:
  left right middle forward back       mouse buttons
  dpi-loop  wheel-up  wheel-down      DPI cycle, wheel
  rgb-toggle  off                      LED on/off, disable button
  key:<hid>                            one keyboard key, HID usage in hex (key:07 = D)
  combo:<mod>:<hid>                    modifier mask + key (combo:08:07 = Win+D)
  raw:<8 hex digits>                   any 4-byte slot value";

const NAMED: &[(&str, [u8; 4])] = &[
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

fn parse_action(s: &str) -> Option<[u8; 4]> {
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

fn describe(v: [u8; 4]) -> String {
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

fn mode_table(arg: Option<&String>) -> Result<(char, u8), String> {
    match arg.map(String::as_str) {
        None | Some("a") => Ok(('A', 1)),
        Some("b") => Ok(('B', 3)),
        Some(m) => Err(format!("unknown mode '{m}', use a or b")),
    }
}

fn print_table(mode: char, t: &ButtonTable) {
    println!("Mode {mode}:");
    for (i, slot) in SLOT.iter().enumerate() {
        println!("  {:>2}  {}", i + 1, describe(t[*slot]));
    }
}

fn run(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("show") => {
            let (mode, table) = mode_table(args.get(1))?;
            let mut mouse = Mouse::open().map_err(|e| e.to_string())?;
            print_table(mode, &mouse.read_buttons(table).map_err(|e| e.to_string())?);
            Ok(())
        }
        Some("set") if args.len() >= 3 => {
            let button: usize = args[1].parse().ok().filter(|b| (1..=10).contains(b))
                .ok_or("button must be 1-10")?;
            let action = parse_action(&args[2]).ok_or(format!("unknown action '{}'", args[2]))?;
            let (mode, table) = mode_table(args.get(3))?;
            let mut mouse = Mouse::open().map_err(|e| e.to_string())?;
            let mut slots = mouse.read_buttons(table).map_err(|e| e.to_string())?;
            slots[SLOT[button - 1]] = action;
            mouse.write_buttons(table, &slots).map_err(|e| e.to_string())?;
            let check = mouse.read_buttons(table).map_err(|e| e.to_string())?;
            if check != slots {
                return Err("the mouse did not store the new table".into());
            }
            print_table(mode, &check);
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
