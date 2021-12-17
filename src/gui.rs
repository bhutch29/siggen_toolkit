use crate::cli::SimulatedChannel;
use crate::logging::{Bool, Level, Logger, LoggingConfiguration, Sink};
use crate::{common, hwconfig, logging};
use clipboard::ClipboardProvider;
use eframe::epi::egui::Color32;
pub use eframe::{egui, egui::Button, egui::CtxRef, egui::Ui, epi};
use image;
use std::fs;
use std::path::PathBuf;
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(PartialEq, EnumIter, Display)]
enum Tabs {
    Logging,
    #[strum(serialize = "Hardware Configuration")]
    HwConfig,
    Versions,
}

struct HwconfigState {
    channel_count: u8,
    platform: SimulatedChannel,
    write_error: bool,
    remove_error: bool,
}

struct LoggingState {
    config: LoggingConfiguration,
    custom_path: String,
    loaded_from: Option<PathBuf>,
    write_error: bool,
    remove_error: bool,
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
                write_error: false,
                remove_error: false,
            },
            logger: LoggingState {
                config: Default::default(),
                loaded_from: None,
                custom_path: String::default(),
                write_error: false,
                remove_error: false,
            },
            selected_tab: Tabs::Logging,
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
            Tabs::Versions => {}
        });
    }

    fn setup(
        &mut self,
        _ctx: &CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.logger.config = logging::get_config_from(&logging::get_path());
        self.logger.loaded_from = Some(logging::get_path());
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
        ui.heading("Hardware Configuration");
        ui.strong("Paths indexed by SigGen:");
        for path in hwconfig::valid_paths().iter() {
            self.hwconfig_path(ui, path);
        }
        ui.strong("Current working directory:");
        self.hwconfig_path(ui, &common::in_cwd(hwconfig::file_name()));

        ui.heading("Simulated Hardware Configuration");
        self.platform_dropdown(ui);
        self.channel_count_selector(ui);
        if let SimulatedChannel::MCS31 { .. } = self.hwconfig.platform {
            self.signal_count_selector(ui);
        }
        ui.add_enabled(
            false,
            egui::TextEdit::multiline(&mut hwconfig::serialize_hwconfig(
                self.hwconfig.platform,
                self.hwconfig.channel_count,
            )),
        );
    }

    fn hwconfig_path(&mut self, ui: &mut Ui, path: &PathBuf) {
        ui.horizontal(|ui| {
            copyable_path(ui, path);
            self.hwconfig_path_buttons(ui, path);
        });
    }

    fn hwconfig_path_buttons(&mut self, ui: &mut Ui, path: &PathBuf) {
        if ui.button("Save").clicked() {
            self.hwconfig.write_error =
                hwconfig::set(path, self.hwconfig.platform, self.hwconfig.channel_count).is_err();
            self.hwconfig.remove_error = false;
        }
        if ui
            .add_enabled(path.exists() && path.is_file(), Button::new("Delete"))
            .clicked()
        {
            self.hwconfig.write_error = false;
            self.hwconfig.remove_error = fs::remove_file(path).is_err();
        }

        if self.hwconfig.write_error {
            error_label(ui, "Error writing configuration to file")
        }
        if self.hwconfig.remove_error {
            error_label(ui, "Error removing file")
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
        ui.heading("KSF Logger Configuration");
        ui.strong("Paths indexed by SigGen:");
        for path in logging::valid_paths().iter() {
            self.logging_path(ui, path);
        }
        ui.strong("Current working directory:");
        self.logging_path(ui, &common::in_cwd(logging::file_name()));
        ui.strong("Custom:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.logger.custom_path);
            self.logging_path_buttons(ui, &PathBuf::from(&self.logger.custom_path));
        });
        ui.separator();

        ui.columns(2, |columns| {
            columns[0].heading("Sinks");
            columns[0].horizontal_wrapped(|ui| {
                ui.label("Create new Sink:");
                for sink in Sink::iter() {
                    if ui.button(sink.to_string()).clicked() {
                        self.logger.config.sinks.push(sink);
                    }
                }
            });

            egui::ScrollArea::vertical()
                .id_source("scroll_sinks")
                .show(&mut columns[0], |ui| {
                    let (sinks_to_remove, sinks_to_add_to_loggers) = self.sinks(ui);

                    for index in sinks_to_remove {
                        self.logger.config.sinks.remove(index);
                    }

                    for sink in sinks_to_add_to_loggers {
                        for logger in self.logger.config.loggers.iter_mut() {
                            logger.sinks.push(sink.clone());
                        }
                    }
                });

            columns[1].heading("Loggers");
            columns[1].horizontal(|ui| {
                ui.label("Create new Logger:");
                if ui.button(" + ").clicked() {
                    self.logger.config.loggers.push(Logger::default());
                }
            });

            egui::ScrollArea::vertical()
                .id_source("scroll_loggers")
                .show(&mut columns[1], |ui| {
                    let loggers_to_remove = self.loggers(ui);
                    for index in loggers_to_remove {
                        self.logger.config.loggers.remove(index);
                    }
                });
        });
    }

    fn logging_path(&mut self, ui: &mut Ui, path: &PathBuf) {
        ui.horizontal(|ui| {
            copyable_path(ui, path);
            self.logging_path_buttons(ui, path);
        });
    }

    fn logging_path_buttons(&mut self, ui: &mut Ui, path: &PathBuf) {
        ui.label(if Some(path) == self.logger.loaded_from.as_ref() {
            "⬅"
        } else {
            "     "
        });

        let exists = path.exists() && path.is_file();
        if ui.add_enabled(exists, Button::new("Load")).clicked() {
            self.logger.config = logging::get_config_from(path);
            self.logger.loaded_from = Some(path.clone());
        }
        if ui.button("Save").clicked() {
            self.logger.remove_error = false;
            self.logger.write_error =
                logging::set_config(path, self.logger.config.clone()).is_err();
            if !self.logger.write_error {
                self.logger.loaded_from = Some(path.clone());
            }
        }
        if ui.add_enabled(exists, Button::new("Delete")).clicked() {
            self.logger.write_error = false;
            self.logger.remove_error = fs::remove_file(path).is_err();
            if Some(path) == self.logger.loaded_from.as_ref() {
                self.logger.loaded_from = None;
            }
        }

        if self.logger.write_error {
            error_label(ui, "Error writing configuration to file");
        }
        if self.logger.remove_error {
            error_label(ui, "Error removing file");
        }
    }

    fn loggers(&mut self, ui: &mut Ui) -> Vec<usize> {
        let mut loggers_to_remove = vec![];
        for (i, logger) in self.logger.config.loggers.iter_mut().enumerate() {
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(" x ").on_hover_text("Remove").clicked() {
                    loggers_to_remove.push(i);
                }
                ui.add(egui::TextEdit::singleline(&mut logger.name).hint_text("Pattern to match"));
            });
            level_dropdown(ui, &mut logger.level, format!("{} {}", &logger.name, i));
            sinks_checkboxes(ui, logger, &self.logger.config.sinks);
        }
        loggers_to_remove
    }

    fn sinks(&mut self, ui: &mut Ui) -> (Vec<usize>, Vec<String>) {
        let mut sinks_to_remove = vec![];
        let mut sinks_to_add_to_loggers = vec![];

        for (i, sink) in self.logger.config.sinks.iter_mut().enumerate() {
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(" x ").on_hover_text("Remove").clicked() {
                    sinks_to_remove.push(i);
                }
                if ui
                    .button(" ✅ ")
                    .on_hover_text("Enable on all loggers")
                    .clicked()
                {
                    sinks_to_add_to_loggers.push(sink.get_name().clone());
                }
                ui.strong(sink.to_string());
            });

            let (name, level) = sink.get_name_and_level_as_mut();
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.add(egui::TextEdit::singleline(name).hint_text("Unique name required"));
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
                    let mut color = logging::is_true(is_color);
                    ui.checkbox(&mut color, "Color");
                    *is_color = Some(Bool::Boolean(color));
                }
                Sink::Etw {
                    ref mut activities_only,
                    ..
                } => {
                    let mut temp = logging::is_true(activities_only);
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
        (sinks_to_remove, sinks_to_add_to_loggers)
    }
}

fn text_edit_labeled(ui: &mut Ui, label: &str, file_name: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(file_name);
    });
}

fn error_label(ui: &mut Ui, label: &str) {
    ui.colored_label(Color32::from_rgb(255, 0, 0), label);
}

fn copyable_path(ui: &mut Ui, path: &PathBuf) {
    if ui
        .selectable_label(false, &path.to_string_lossy())
        .on_hover_text("Click to copy")
        .clicked()
    {
        if let Ok(mut clip) = clipboard::ClipboardContext::new() {
            let _ = clip.set_contents(path.to_string_lossy().to_string());
        }
    }
}

fn truncate_ui(ui: &mut Ui, truncate: &mut Option<Bool>) {
    let mut trunc = logging::is_true(truncate);
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
    }

    remove_invalid_sinks(logger, sinks);
}

fn remove_invalid_sinks(logger: &mut Logger, sinks: &Vec<Sink>) {
    logger.sinks.retain(|logger_sink_name| {
        sinks
            .iter()
            .any(|target_sink| target_sink.get_name() == logger_sink_name)
    });
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

    let icon = image::open("keysight-logo.ico")
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    let options = eframe::NativeOptions {
        icon_data: Some(eframe::epi::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };

    eframe::run_native(Box::new(app), options);
}
