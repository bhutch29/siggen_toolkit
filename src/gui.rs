use crate::cli::SimulatedChannel;
use crate::hwconfig;
use crate::logging::*;
pub use eframe::{egui, egui::CtxRef, egui::Ui, epi};
use std::fs;

use clipboard::ClipboardProvider;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(PartialEq, EnumIter, Display)]
enum Tabs {
    #[strum(serialize = "Hardware Configuration")]
    HwConfig,
    Logging,
}

struct HwconfigState {
    channel_count: u8,
    platform: SimulatedChannel,
    text: Option<String>,
}

struct LoggingState {
    config: Option<LoggingConfiguration>,
}

struct GuiApp {
    selected_tab: Tabs,
    hwconfig: HwconfigState,
    logger: LoggingState,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            hwconfig: HwconfigState {
                channel_count: 1,
                platform: SimulatedChannel::MCS31 { signal_count: 1 },
                text: None,
            },
            logger: LoggingState { config: None },
            selected_tab: Tabs::HwConfig,
        }
    }
}

impl epi::App for GuiApp {
    fn update(&mut self, ctx: &CtxRef, frame: &mut epi::Frame<'_>) {
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
        _ctx: &CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.hwconfig.text = hwconfig::read();

        let logging_text =
            fs::read_to_string("./ksflogger.cfg").expect("failed to read ksflogger.cfg");
        self.logger.config =
            serde_json::from_str(&logging_text).expect("failed to deserialize ksflogger.cfg");
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
        if let SimulatedChannel::MCS31 { .. } = self.hwconfig.platform {
            self.signal_count_selector(ui);
        }
        if ui.button("Apply Simulated Config").clicked() {
            hwconfig::set(self.hwconfig.platform, self.hwconfig.channel_count);
            self.hwconfig.text = hwconfig::read();
        }

        if self.hwconfig.text.is_some() {
            ui.separator();
            ui.heading("Current Hardware Configuration");
            ui.text_edit_multiline(self.hwconfig.text.as_mut().unwrap());
            if ui.button("Write Configuration").clicked() {
                fs::write(hwconfig::get_path(), self.hwconfig.text.as_ref().unwrap())
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
            ui.selectable_value(&mut self.hwconfig.channel_count, 1, "1");
            ui.selectable_value(&mut self.hwconfig.channel_count, 2, "2");
            ui.selectable_value(&mut self.hwconfig.channel_count, 4, "4");
            ui.label("Number of Channels");
        });
    }

    fn signal_count_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.hwconfig.platform,
                SimulatedChannel::MCS31 { signal_count: 1 },
                "1",
            );
            ui.selectable_value(
                &mut self.hwconfig.platform,
                SimulatedChannel::MCS31 { signal_count: 8 },
                "8",
            );
            ui.label("Signals per Channel");
        });
    }

    fn platform_dropdown(&mut self, ui: &mut Ui) {
        egui::ComboBox::from_label("Platform")
            .selected_text(format!("{}", self.hwconfig.platform))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.hwconfig.platform,
                    SimulatedChannel::MCS31 { signal_count: 0 },
                    "MCS3.1",
                );
                ui.selectable_value(
                    &mut self.hwconfig.platform,
                    SimulatedChannel::MCS15,
                    "MCS1.5",
                );
            });
    }

    fn logging(&mut self, ui: &mut Ui) {
        let config = &mut self.logger.config.as_mut().unwrap();
        let mut sinks_to_remove: Vec<usize> = vec![];
        let mut loggers_to_remove: Vec<usize> = vec![];

        ui.columns(2, |columns| {
            columns[0].heading("Sinks");
            columns[0].horizontal_wrapped(|ui| {
                ui.label("Create New Sink:");
                for sink in Sink::iter() {
                    if ui.button(sink.to_string()).clicked() {
                        config.sinks.push(sink);
                    }
                }
            });

            egui::ScrollArea::vertical()
                .id_source("scroll_sinks")
                .show(&mut columns[0], |ui| {
                    sinks_to_remove = sinks(ui, config);
                });

            columns[1].heading("Loggers");
            egui::ScrollArea::vertical()
                .id_source("scroll_loggers")
                .show(&mut columns[1], |ui| {
                    loggers_to_remove = loggers(ui, config);
                });
        });

        for index in sinks_to_remove {
            config.sinks.remove(index);
        }

        for index in loggers_to_remove {
            config.loggers.remove(index);
        }
    }
}

fn loggers(ui: &mut Ui, config: &mut LoggingConfiguration) -> Vec<usize> {
    let mut loggers_to_remove = vec![];
    for (i, logger) in config.loggers.iter_mut().enumerate() {
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(" - ").on_hover_text("Remove Logger").clicked() {
                loggers_to_remove.push(i);
            }
            ui.text_edit_singleline(&mut logger.name);
        });
        level_dropdown(ui, &mut logger.level, format!("{} {}", &logger.name, i));
        sinks_checkboxes(ui, logger, &config.sinks);
    }
    loggers_to_remove
}

fn sinks(ui: &mut Ui, config: &mut LoggingConfiguration) -> Vec<usize> {
    let mut sinks_to_remove = vec![];
    for (i, sink) in config.sinks.iter_mut().enumerate() {
        ui.separator();

        ui.horizontal(|ui| {
            if ui.button(" x ").on_hover_text("Remove Sink").clicked() {
                sinks_to_remove.push(i);
            }
            ui.strong(sink.to_string());
        });

        let (name, level) = sink.get_name_and_level_as_mut();
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(name);
        });

        level_dropdown(ui, level, format!("{} {}", name, i));

        match sink {
            Sink::RotatingFile {
                ref mut file_name,
                ref mut truncate,
                ref mut max_size,
                ref mut max_files,
                ..
            } => {
                text_edit_labeled(ui, "File Path", file_name);
                truncate_ui(ui, truncate);

                let mut files = max_files.unwrap_or_default();
                ui.add(egui::Slider::new(&mut files, 0..=50).text("Max Files"));
                *max_files = Some(files);

                let mut size = max_size.unwrap_or_default();
                ui.add(egui::Slider::new(&mut size, 0..=5_000_000).text("Max Size"));
                *max_size = Some(size);
            }
            Sink::File {
                ref mut file_name,
                ref mut truncate,
                ..
            } => {
                text_edit_labeled(ui, "File Path", file_name);
                truncate_ui(ui, truncate);
            }
            Sink::DailyFile {
                ref mut file_name,
                ref mut truncate,
                ..
            } => {
                text_edit_labeled(ui, "File Path", file_name);
                truncate_ui(ui, truncate);
            }
            Sink::Console {
                ref mut is_color, ..
            } => {
                let mut color = is_true(is_color);
                ui.checkbox(&mut color, "Color");
                *is_color = Some(Bool::Boolean(color));
            }
            Sink::Etw {
                ref mut activities_only,
                ..
            } => {
                let mut temp = is_true(activities_only);
                ui.checkbox(&mut temp, "Activities Only");
                *activities_only = Some(Bool::Boolean(temp));
            }
            Sink::Windiag { .. } => {}
            Sink::EventLog { .. } => {}
            Sink::Nats { ref mut url, .. } => {
                ui.horizontal(|ui| {
                    ui.label("Url");
                    ui.text_edit_singleline(url);
                });
            }
        }
    }
    sinks_to_remove
}

fn text_edit_labeled(ui: &mut Ui, label: &str, file_name: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(file_name);
    });
}

fn truncate_ui(ui: &mut Ui, truncate: &mut Option<Bool>) {
    let mut trunc = is_true(truncate);
    ui.checkbox(&mut trunc, "Truncate");
    *truncate = Some(Bool::Boolean(trunc));
}

fn sinks_checkboxes(ui: &mut Ui, logger: &mut Logger, sinks: &Vec<Sink>) {
    for sink in sinks {
        let name = sink.get_name();
        let mut checked = logger.sinks.contains(name);
        ui.checkbox(&mut checked, name);
        if checked && !logger.sinks.contains(name) {
            logger.sinks.push(name.clone());
        }
        if !checked && logger.sinks.contains(name) {
            logger.sinks.retain(|x| x != name);
        }

        logger.sinks.retain(|logger_sink_name| {
            sinks
                .iter()
                .any(|target_sink| target_sink.get_name() == logger_sink_name)
        });
    }
}

fn level_dropdown(ui: &mut Ui, level: &mut Level, id: impl std::hash::Hash) {
    egui::ComboBox::from_id_source(id)
        .selected_text(format!("{}", level))
        .show_ui(ui, |ui| {
            for option in Level::iter() {
                ui.selectable_value(level, option, option.to_string());
            }
        });
}

pub fn run() {
    let app = GuiApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
