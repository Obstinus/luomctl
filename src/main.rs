use luomctl::device::{ButtonTable, Mouse};
use luomctl::{describe, parse_action, SLOT};
use std::process::ExitCode;

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
