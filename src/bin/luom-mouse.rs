//! Simple window to change the mouse buttons.

use eframe::egui;
use luomctl::device::{ButtonTable, Mouse};
use luomctl::*;
use std::io::ErrorKind;

/// Button names. The numbers are the same as in the official software.
const BUTTONS: [&str; 10] = [
    "1 · Left button",
    "2 · Right button",
    "3 · Wheel click",
    "4 · Extra button 4",
    "5 · Extra button 5",
    "6 · Extra button 6",
    "7 · Extra button 7",
    "8 · Thumb button, upper",
    "9 · Thumb button, lower",
    "10 · Extra button 10",
];

/// The actions a person can choose, in plain words.
const CHOICES: &[(&str, [u8; 4])] = &[
    ("Left click", LEFT),
    ("Right click", RIGHT),
    ("Middle click", MIDDLE),
    ("Mouse 4 (back)", BACK),
    ("Mouse 5 (forward)", FORWARD),
    ("Scroll up", WHEEL_UP),
    ("Scroll down", WHEEL_DOWN),
    ("Change speed (DPI)", DPI_LOOP),
    ("Lights on / off", RGB_TOGGLE),
    ("Show desktop (Win+D)", [0x00, 0x08, 0x07, 0x00]),
    ("Copy (Ctrl+C)", [0x00, 0x01, 0x06, 0x00]),
    ("Paste (Ctrl+V)", [0x00, 0x01, 0x19, 0x00]),
    ("Do nothing", OFF),
];

fn label(v: [u8; 4]) -> String {
    CHOICES
        .iter()
        .find(|(_, x)| *x == v)
        .map(|(n, _)| n.to_string())
        .unwrap_or_else(|| "Special (set in other software)".into())
}

fn friendly(e: std::io::Error) -> String {
    match e.kind() {
        ErrorKind::NotFound => "The mouse is not connected. Connect it and click “Try again”.".into(),
        ErrorKind::PermissionDenied => {
            "This computer does not let the program use the mouse. Ask a helper to install the file 70-luom-g10.rules (see README).".into()
        }
        _ => format!("The mouse did not answer. Disconnect it, connect it again, and click “Try again”. ({e})"),
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    A = 1,
    B = 3,
}

struct App {
    mode: Mode,
    saved: Option<ButtonTable>,
    edit: ButtonTable,
    message: String,
    ok: bool,
}

impl App {
    fn new() -> Self {
        let mut app = App { mode: Mode::A, saved: None, edit: [[0; 4]; 16], message: String::new(), ok: true };
        app.load();
        app
    }

    fn load(&mut self) {
        match Mouse::open().and_then(|mut m| m.read_buttons(self.mode as u8)) {
            Ok(t) => {
                self.saved = Some(t);
                self.edit = t;
                self.message = "Mouse found. Choose what each button does, then click “Save to mouse”.".into();
                self.ok = true;
            }
            Err(e) => {
                self.saved = None;
                self.message = friendly(e);
                self.ok = false;
            }
        }
    }

    fn save(&mut self) {
        let mode = self.mode as u8;
        let result = Mouse::open().and_then(|mut m| {
            m.write_buttons(mode, &self.edit)?;
            m.read_buttons(mode)
        });
        match result {
            Ok(t) if t == self.edit => {
                self.saved = Some(t);
                self.message = "Saved. The mouse keeps these settings, also on other computers.".into();
                self.ok = true;
            }
            Ok(t) => {
                self.saved = Some(t);
                self.message = "The mouse did not keep the change. Click “Save to mouse” again.".into();
                self.ok = false;
            }
            Err(e) => {
                self.message = friendly(e);
                self.ok = false;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.add_space(4.0);
            let color = if self.ok { egui::Color32::from_rgb(40, 150, 70) } else { egui::Color32::from_rgb(200, 60, 50) };
            ui.colored_label(color, &self.message);
            ui.add_space(4.0);
        });
        egui::SidePanel::left("picture").resizable(false).show(ctx, |ui| {
            ui.add_space(30.0);
            draw_mouse(ui);
            ui.label("The numbers on the picture\nare the button numbers.");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mouse buttons");
            ui.add_space(6.0);

            let changed = self.saved.is_some_and(|saved| saved != self.edit);
            ui.horizontal(|ui| {
                ui.label("Button set:");
                let before = self.mode;
                ui.add_enabled_ui(!changed, |ui| {
                    ui.selectable_value(&mut self.mode, Mode::A, "Set A");
                    ui.selectable_value(&mut self.mode, Mode::B, "Set B");
                });
                if self.mode != before {
                    self.load();
                }
            })
            .response
            .on_hover_text(if changed {
                "Save or undo your changes before you change the set."
            } else {
                "The mouse can keep two sets. A button on the mouse changes between them."
            });
            ui.add_space(8.0);

            if let Some(saved) = self.saved {
                egui::Grid::new("buttons").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                    for (i, name) in BUTTONS.iter().enumerate() {
                        ui.label(*name);
                        let slot = &mut self.edit[SLOT[i]];
                        egui::ComboBox::from_id_salt(i)
                            .width(240.0)
                            .selected_text(label(*slot))
                            .show_ui(ui, |ui| {
                                for (text, value) in CHOICES {
                                    ui.selectable_value(slot, *value, *text);
                                }
                            });
                        ui.end_row();
                    }
                });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.add_enabled(changed, egui::Button::new("Save to mouse")).clicked() {
                        self.save();
                    }
                    if ui.add_enabled(changed, egui::Button::new("Undo changes")).clicked() {
                        self.edit = saved;
                    }
                });
            } else if ui.button("Try again").clicked() {
                self.load();
            }
        });
    }
}

/// Picture of the mouse from above, with the button numbers at the same places as the official software.
fn draw_mouse(ui: &mut egui::Ui) {
    let svg = egui::Image::from_bytes("bytes://mouse.svg", include_bytes!("../../assets/mouse.svg").as_slice());
    let rect = ui.add(svg.fit_to_exact_size(egui::vec2(280.0, 415.0))).rect;
    let scale = rect.width() / 560.0;
    let numbers = [(160.0, 265.0), (400.0, 240.0), (280.0, 350.0), (455.0, 330.0), (200.0, 415.0),
                   (360.0, 415.0), (75.0, 400.0), (75.0, 565.0), (85.0, 680.0), (510.0, 420.0)];
    for (n, (x, y)) in numbers.into_iter().enumerate() {
        let at = rect.min + egui::vec2(x, y) * scale;
        ui.painter().circle(at, 11.0, egui::Color32::WHITE, egui::Stroke::new(1.5_f32, egui::Color32::BLACK));
        ui.painter().text(at, egui::Align2::CENTER_CENTER, (n + 1).to_string(), egui::FontId::proportional(13.0), egui::Color32::BLACK);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 600.0])
            .with_min_inner_size([720.0, 520.0])
            .with_title("Mouse buttons"),
        ..Default::default()
    };
    eframe::run_native("Mouse buttons", options, Box::new(|cc| {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        Ok(Box::new(App::new()))
    }))
}
