use luomctl::device::{ButtonTable, Mouse};
use luomctl::{describe, light_label, parse_action, ReportRate, LIGHTS, SLOT};
use std::process::ExitCode;

const USAGE: &str = "\
luomctl - configure the LUOM G10 / Combaterwing mouse (04d9:a09f)

USAGE:
  luomctl show [a|b]                     show the button actions
  luomctl set <button> <action> [a|b]    change one button (default: mode a)
  luomctl rate [125|250|500|1000]        show or set the report rate in Hz
  luomctl light [name]                   show or set the light effect

BUTTON: 1-10, as numbered in the official software

ACTION:
  left right middle forward back       mouse buttons
  dpi-loop  wheel-up  wheel-down      DPI cycle, wheel
  rgb-toggle  off                      LED on/off, disable button
  key:<hid>                            one keyboard key, HID usage in hex (key:07 = D)
  combo:<mod>:<hid>                    modifier mask + key (combo:08:07 = Win+D)
  raw:<8 hex digits>                   any 4-byte slot value

LIGHT:
  breathing   breathing, colour cycle
  flashing    breathing, flashing
  steady      steady colour";

fn parse_rate(arg: &str) -> Result<ReportRate, String> {
    ReportRate::ALL
        .into_iter()
        .find(|r| r.hz().to_string() == arg)
        .ok_or_else(|| format!("rate must be 125, 250, 500 or 1000, not '{arg}'"))
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
        Some("rate") if args.len() == 1 => {
            let mouse = Mouse::open().map_err(|e| e.to_string())?;
            let rate = mouse.read_rate().map_err(|e| e.to_string())?;
            println!("Report rate: {} Hz", rate.hz());
            Ok(())
        }
        Some("rate") if args.len() == 2 => {
            let rate = parse_rate(&args[1])?;
            let mouse = Mouse::open().map_err(|e| e.to_string())?;
            mouse.write_rate(rate).map_err(|e| e.to_string())?;
            let check = mouse.read_rate().map_err(|e| e.to_string())?;
            if check != rate {
                return Err("the mouse did not store the report rate".into());
            }
            println!("Report rate: {} Hz", check.hz());
            Ok(())
        }
        Some("light") if args.len() == 1 => {
            let mouse = Mouse::open().map_err(|e| e.to_string())?;
            let state = mouse.read_light().map_err(|e| e.to_string())?;
            println!("Light: {}", light_label(&state));
            Ok(())
        }
        Some("light") if args.len() == 2 => {
            let preset = LIGHTS.iter().find(|p| p.cli == args[1]).ok_or_else(|| {
                let names: Vec<&str> = LIGHTS.iter().map(|p| p.cli).collect();
                format!("unknown light '{}', use {}", args[1], names.join(", "))
            })?;
            let mouse = Mouse::open().map_err(|e| e.to_string())?;
            let want = preset.apply(&mouse.read_light().map_err(|e| e.to_string())?);
            mouse.write_light(&want).map_err(|e| e.to_string())?;
            let check = mouse.read_light().map_err(|e| e.to_string())?;
            if check != want {
                return Err("the mouse did not store the light".into());
            }
            println!("Light: {}", preset.name);
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
