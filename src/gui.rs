use crate::cli::SimulatedChannel;
use crate::hwconfig;
pub use eframe::{egui, egui::Ui, epi};
use std::fs;

use clipboard::ClipboardProvider;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(PartialEq, EnumIter, Display)]
enum Tabs {
    #[strum(serialize = "Hardware Configuration")]
    HwConfig,
    Logging,
}

struct GuiApp {
    channel_count: u8,
    selected_tab: Tabs,
    platform: SimulatedChannel,
    hwconfig_text: Option<String>,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            channel_count: 1,
            selected_tab: Tabs::HwConfig,
            platform: SimulatedChannel::MCS31 { signal_count: 1 },
            hwconfig_text: None,
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
                for tab in Tabs::iter() {
                    self.make_tab(ui, tab);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.selected_tab {
            Tabs::HwConfig => {
                self.hwconfig(ui);
            }
            Tabs::Logging => {
                self.logging(ui);
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
    fn make_tab(&mut self, ui: &mut Ui, tab: Tabs) {
        if ui
            .selectable_label(self.selected_tab == tab, tab.to_string())
            .clicked()
        {
            self.selected_tab = tab;
        }
    }

    fn hwconfig(&mut self, ui: &mut Ui) {
        ui.heading("Hardware Configuration Path");
        self.hwconfig_path(ui);
        ui.separator();

        ui.heading("Simulated Hardware Configuration");
        self.platform_dropdown(ui);
        self.channel_count_selector(ui);
        if let SimulatedChannel::MCS31 { .. } = self.platform {
            self.signal_count_selector(ui);
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
                fs::write(hwconfig::get_path(), self.hwconfig_text.as_ref().unwrap())
                    .expect("Failed to write custom config.")
            }
        }
    }

    fn hwconfig_path(&mut self, ui: &mut Ui) {
        if ui
            .selectable_label(false, hwconfig::get_path().to_str().unwrap())
            .on_hover_text("Click to copy")
            .clicked()
        {
            let mut clip: clipboard::ClipboardContext =
                clipboard::ClipboardProvider::new().unwrap();
            clip.set_contents(hwconfig::get_path().to_string_lossy().to_string())
                .expect("Unable to copy to clipboard");
        }
    }

    fn channel_count_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.channel_count, 1, "1");
            ui.selectable_value(&mut self.channel_count, 2, "2");
            ui.selectable_value(&mut self.channel_count, 4, "4");
            ui.label("Number of Channels");
        });
    }

    fn signal_count_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.platform,
                SimulatedChannel::MCS31 { signal_count: 1 },
                "1",
            );
            ui.selectable_value(
                &mut self.platform,
                SimulatedChannel::MCS31 { signal_count: 8 },
                "8",
            );
            ui.label("Signals per Channel");
        });
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

    fn logging(&mut self, ui: &mut Ui) {
        ui.heading("Logging");
    }
}

pub fn run() {
    let app = GuiApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
