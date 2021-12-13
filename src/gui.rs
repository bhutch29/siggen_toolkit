use crate::cli::SimulatedChannel;
pub use eframe::{egui, epi, egui::Ui};
use std::fmt::{Display, Formatter, Result};
use crate::hwconfig;
use std::fs;

#[derive(PartialEq)]
enum Tabs {
    HwConfig,
    Log,
}

impl Display for Tabs {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
            Tabs::HwConfig => {
                write!(f, "Hardware Configuration")
            }
            Tabs::Log => {
                write! {f, "Logging"}
            }
        }
    }
}

struct GuiApp {
    channel_count: u8,
    selected_tab: Tabs,
    platform: SimulatedChannel,
    hwconfig_text: Option<String>
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            channel_count: 1,
            selected_tab: Tabs::HwConfig,
            platform: SimulatedChannel::MCS31 { signal_count: 1 },
            hwconfig_text: None
        }
    }
}

impl epi::App for GuiApp {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.separator();
                self.make_tab(ui, Tabs::HwConfig);
                self.make_tab(ui, Tabs::Log);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tabs::HwConfig => {
                self.hwconfig(ui);
            }
            Tabs::Log => {
                ui.heading("Second Tab");
            }
        });
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.hwconfig_text = hwconfig::read();
    }

    fn name(&self) -> &str {
        "SigGen Toolkit"
    }
}

impl GuiApp {
    fn hwconfig(&mut self, ui: &mut Ui) {
        ui.heading("Hardware Configuration Path");
        ui.label(hwconfig::get_path().to_str().unwrap());
        ui.separator();

        ui.heading("Simulated Hardware Configuration");
        self.platform_dropdown(ui);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.channel_count, 1, "1");
            ui.selectable_value(&mut self.channel_count, 2, "2");
            ui.selectable_value(&mut self.channel_count, 4, "4");
            ui.label("Number of Channels");
        });
        if let SimulatedChannel::MCS31 { .. } = self.platform {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.platform, SimulatedChannel::MCS31 {signal_count: 1}, "1");
                ui.selectable_value(&mut self.platform, SimulatedChannel::MCS31 {signal_count: 8}, "8");
                ui.label("Signals per Channel");
            });
        }
        if ui.button("Apply Simulated Config").clicked() {
            hwconfig::set(self.platform, self.channel_count);
            self.hwconfig_text = hwconfig::read();
        }

        if self.hwconfig_text.is_some() {
            ui.separator();
            ui.heading("Current Hardware Configuration");
            ui.text_edit_multiline(self.hwconfig_text.as_mut().unwrap());
            if ui.button("Write Configuration").clicked() {
                fs::write(hwconfig::get_path(), self.hwconfig_text.as_ref().unwrap()).expect("Failed to write custom config.")
            }
        }
    }

    fn make_tab(&mut self, ui: &mut Ui, tab: Tabs) {
        if ui
            .selectable_label(self.selected_tab == tab, tab.to_string())
            .clicked()
        {
            self.selected_tab = tab;
        }
    }

    fn platform_dropdown(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_label("Platform")
            .selected_text(format!("{}", self.platform))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.platform,
                    SimulatedChannel::MCS31 { signal_count: 0 },
                    "MCS3.1",
                );
                ui.selectable_value(&mut self.platform, SimulatedChannel::MCS15, "MCS1.5");
            });
    }
}

pub fn run() {
    let app = GuiApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
