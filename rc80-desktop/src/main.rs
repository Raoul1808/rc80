use eframe::egui;

use rc80_core::System;

#[derive(Default)]
struct EmuApp {
    sys: System,
}

impl EmuApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        let bytes = include_bytes!("/home/mew/Downloads/1-chip8-logo.ch8");
        app.sys.load(bytes);
        app
    }
}

impl eframe::App for EmuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello eframe!");
            if ui.button("Step emulation").clicked() {
                self.sys.step();
            }
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "rc80 Desktop",
        native_options,
        Box::new(|cc| Box::new(EmuApp::new(cc))),
    )
    .unwrap()
}
