//! Simple window to change the mouse buttons.

use eframe::egui;
use luomctl::device::{ButtonTable, Mouse};
use luomctl::SLOT;
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
    ("Left click", [0x01, 0x00, 0xf0, 0x00]),
    ("Right click", [0x01, 0x00, 0xf1, 0x00]),
    ("Middle click", [0x01, 0x00, 0xf2, 0x00]),
    ("Mouse 4 (back)", [0x01, 0x00, 0xf3, 0x00]),
    ("Mouse 5 (forward)", [0x01, 0x00, 0xf4, 0x00]),
    ("Scroll up", [0x01, 0x00, 0xf7, 0x00]),
    ("Scroll down", [0x01, 0x00, 0xf8, 0x00]),
    ("Change speed (DPI)", [0x07, 0x00, 0x03, 0x00]),
    ("Lights on / off", [0x08, 0x00, 0x03, 0x00]),
    ("Show desktop (Win+D)", [0x00, 0x08, 0x07, 0x00]),
    ("Copy (Ctrl+C)", [0x00, 0x01, 0x06, 0x00]),
    ("Paste (Ctrl+V)", [0x00, 0x01, 0x19, 0x00]),
    ("Do nothing", [0x00, 0x00, 0x00, 0x00]),
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
            Ok(_) => {
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
        egui::SidePanel::left("picture").resizable(false).show(ctx, |ui| {
            ui.add_space(30.0);
            draw_mouse(ui);
            ui.label("The numbers on the picture\nare the button numbers.");
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Mouse buttons");
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Button set:");
                let before = self.mode;
                ui.selectable_value(&mut self.mode, Mode::A, "Set A");
                ui.selectable_value(&mut self.mode, Mode::B, "Set B");
                if self.mode != before {
                    self.load();
                }
            })
            .response
            .on_hover_text("The mouse can keep two sets. A button on the mouse changes between them.");
            ui.add_space(8.0);

            if self.saved.is_some() {
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
                    let changed = self.saved != Some(self.edit);
                    if ui.add_enabled(changed, egui::Button::new("Save to mouse")).clicked() {
                        self.save();
                    }
                    if ui.add_enabled(changed, egui::Button::new("Undo changes")).clicked() {
                        self.edit = self.saved.unwrap();
                    }
                });
            } else if ui.button("Try again").clicked() {
                self.load();
            }

            ui.add_space(10.0);
            let color = if self.ok { egui::Color32::from_rgb(40, 150, 70) } else { egui::Color32::from_rgb(200, 60, 50) };
            ui.colored_label(color, &self.message);
        });
    }
}

/// Draw the mouse from above, with the button numbers at the same places as the official software.
fn draw_mouse(ui: &mut egui::Ui) {
    use egui::{pos2, Color32, Shape, Stroke};
    let (rect, _) = ui.allocate_exact_size(egui::vec2(245.0, 300.0), egui::Sense::hover());
    let p = |x: f32, y: f32| rect.min + egui::vec2(x, y);
    let painter = ui.painter_at(rect);
    let shell = Color32::from_rgb(35, 35, 40);
    let edge = Stroke::new(2.0_f32, Color32::from_rgb(200, 160, 70));
    let red = Color32::from_rgb(190, 30, 35);

    let poly = |pts: &[(f32, f32)], fill: Color32| {
        Shape::convex_polygon(pts.iter().map(|&(x, y)| p(x, y)).collect(), fill, edge)
    };
    // Body, palm rest, two main buttons, thumb panel.
    painter.add(Shape::closed_line(
        [(55.0, 40.0), (190.0, 40.0), (215.0, 120.0), (210.0, 220.0), (160.0, 290.0), (85.0, 290.0), (25.0, 220.0), (22.0, 120.0)]
            .iter().map(|&(x, y)| p(x, y)).collect(),
        edge,
    ));
    painter.add(poly(&[(35.0, 210.0), (205.0, 210.0), (160.0, 288.0), (85.0, 288.0)], shell));
    painter.add(poly(&[(55.0, 45.0), (112.0, 45.0), (112.0, 165.0), (100.0, 200.0), (40.0, 200.0), (30.0, 120.0)], shell));
    painter.add(poly(&[(140.0, 45.0), (190.0, 45.0), (210.0, 120.0), (200.0, 200.0), (152.0, 200.0), (140.0, 165.0)], shell));
    painter.add(poly(&[(10.0, 135.0), (28.0, 125.0), (34.0, 280.0), (14.0, 270.0)], Color32::from_rgb(60, 25, 25)));
    // Wheel.
    painter.rect(egui::Rect::from_min_max(p(116.0, 80.0), p(136.0, 125.0)), 8.0, red, edge, egui::StrokeKind::Middle);
    for y in [90.0f32, 100.0, 110.0] {
        painter.line_segment([p(119.0, y), p(133.0, y)], Stroke::new(1.0_f32, Color32::BLACK));
    }
    painter.circle_filled(pos2(rect.center().x, rect.min.y + 250.0), 10.0, red);

    for (n, x, y) in [(1, 65.0f32, 125.0f32), (2, 165.0, 70.0), (3, 126.0, 145.0), (4, 180.0, 110.0), (5, 82.0, 180.0),
                      (6, 168.0, 185.0), (7, 40.0, 150.0), (8, 22.0, 195.0), (9, 24.0, 250.0), (10, 200.0, 145.0)] {
        painter.circle(p(x, y), 10.0, Color32::WHITE, Stroke::new(1.0_f32, Color32::BLACK));
        painter.text(p(x, y), egui::Align2::CENTER_CENTER, n.to_string(), egui::FontId::proportional(12.0), Color32::BLACK);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 540.0]).with_title("Mouse buttons"),
        ..Default::default()
    };
    eframe::run_native("Mouse buttons", options, Box::new(|_| Ok(Box::new(App::new()))))
}
