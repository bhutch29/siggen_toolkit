use crate::cli::SimulatedChannel;
use crate::hwconfig;
use crate::logging;
pub use eframe::{egui, egui::Ui, epi};
use std::fs;

use crate::logging::Sink;
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
    ksflogger_config: Option<logging::LoggingConfiguration>,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            channel_count: 1,
            selected_tab: Tabs::HwConfig,
            platform: SimulatedChannel::MCS31 { signal_count: 1 },
            hwconfig_text: None,
            ksflogger_config: None,
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

        let logging_text =
            fs::read_to_string("./ksflogger.cfg").expect("failed to read ksflogger.cfg");
        self.ksflogger_config =
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
        let config = &mut self.ksflogger_config.as_mut().unwrap();
        let mut sinks_to_remove = vec![];
        let mut loggers_to_remove = vec![];

        ui.columns(2, |columns| {
            columns[0].heading("Sinks");
            columns[0].horizontal_wrapped(|ui| {
                ui.label("Create New Sink:");
                for sink in logging::Sink::iter() {
                    if ui.button(sink.to_string()).clicked() {
                        config.sinks.push(sink);
                    }
                }
            });

            egui::ScrollArea::vertical()
                .id_source("sinks scrolling")
                .show(&mut columns[0], |ui| {
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

                        GuiApp::level_dropdown(ui, level, format!("{} {}", name, i));

                        let mut file_name_ui = |file_name| {
                            ui.horizontal(|ui| {
                                ui.label("File Path");
                                ui.text_edit_singleline(file_name);
                            });
                        };

                        match sink {
                            Sink::RotatingFile {
                                ref mut file_name,
                                ref mut truncate,
                                ref mut max_size,
                                ref mut max_files,
                                ..
                            } => {
                                file_name_ui(file_name);

                                let mut trunc = logging::is_true(truncate);
                                ui.checkbox(&mut trunc, "Truncate");
                                *truncate = Some(logging::Bool::Boolean(trunc));

                                {
                                    let mut temp = max_files.unwrap_or_default();
                                    ui.add(egui::Slider::new(&mut temp, 0..=50).text("Max Files"));
                                    *max_files = Some(temp);
                                }

                                {
                                    let mut temp = max_size.unwrap_or_default();
                                    ui.add(
                                        egui::Slider::new(&mut temp, 0..=5_000_000)
                                            .text("Max Size"),
                                    );
                                    *max_size = Some(temp);
                                }
                            }
                            Sink::File {
                                ref mut file_name,
                                ref mut truncate,
                                ..
                            } => {
                                file_name_ui(file_name);

                                let mut trunc = logging::is_true(truncate);
                                ui.checkbox(&mut trunc, "Truncate");
                                *truncate = Some(logging::Bool::Boolean(trunc));
                            }
                            Sink::DailyFile {
                                ref mut file_name,
                                ref mut truncate,
                                ..
                            } => {
                                file_name_ui(file_name);

                                let mut trunc = logging::is_true(truncate);
                                ui.checkbox(&mut trunc, "Truncate");
                                *truncate = Some(logging::Bool::Boolean(trunc));
                            }
                            Sink::Console {
                                ref mut is_color, ..
                            } => {
                                let mut color = logging::is_true(is_color);
                                ui.checkbox(&mut color, "Color");
                                *is_color = Some(logging::Bool::Boolean(color));
                            }
                            Sink::Etw {
                                ref mut activities_only,
                                ..
                            } => {
                                let mut temp = logging::is_true(activities_only);
                                ui.checkbox(&mut temp, "Activities Only");
                                *activities_only = Some(logging::Bool::Boolean(temp));
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
                });

            columns[1].heading("Loggers");
            egui::ScrollArea::vertical()
                .id_source("loggers scrolling")
                .show(&mut columns[1], |ui| {
                    let sinks = config.sinks.clone();
                    for (i, logger) in config.loggers.iter_mut().enumerate() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button(" - ").on_hover_text("Remove Logger").clicked() {
                                loggers_to_remove.push(i);
                            }
                            ui.text_edit_singleline(&mut logger.name);
                        });
                        GuiApp::level_dropdown(ui, &mut logger.level, format!("{} {}", &logger.name, i));
                        GuiApp::sinks_checkboxes(ui, logger, &sinks);
                    }
                });
        });

        for index in sinks_to_remove {
            config.sinks.remove(index);
        }

        for index in loggers_to_remove {
            config.loggers.remove(index);
        }
    }

    fn sinks_checkboxes(ui: &mut Ui, logger: &mut logging::Logger, sinks: &Vec<logging::Sink>) {
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

    fn level_dropdown(ui: &mut Ui, level: &mut logging::Level, id: impl std::hash::Hash) {
        egui::ComboBox::from_id_source(id)
            .selected_text(format!("{}", level))
            .show_ui(ui, |ui| {
                for option in logging::Level::iter() {
                    ui.selectable_value(level, option, option.to_string());
                }
            });
    }
}

pub fn run() {
    let app = GuiApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
