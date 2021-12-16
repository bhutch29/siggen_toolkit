use crate::cli::SimulatedChannel;
use crate::{hwconfig, logging};
use crate::logging::{Bool, Level, Logger, LoggingConfiguration, Sink};
pub use eframe::{egui, egui::CtxRef, egui::Ui, epi};
use std::fs;
use image;
use clipboard::ClipboardProvider;
use strum::{Display, EnumIter, IntoEnumIterator};
use eframe::epi::egui::Color32;
use std::path::PathBuf;

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
    write_error: bool,
}

struct LoggingState {
    config: LoggingConfiguration,
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
                write_error: false,
            },
            logger: LoggingState {
                config: Default::default(),
            },
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
        self.logger.config = logging::get_current_config();
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
        clickable_path(ui, hwconfig::get_path());
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
            self.hwconfig.write_error = false; // cleanup any old errors
        }

        if let Some(ref mut text) = self.hwconfig.text {
            ui.separator();
            ui.heading("Current Hardware Configuration");
            ui.text_edit_multiline(text);
            if ui.button("Write Configuration").clicked() {
                self.hwconfig.write_error = fs::write(hwconfig::get_path(), text).is_err();
            }

            if self.hwconfig.write_error {
                ui.colored_label(Color32::from_rgb(255, 0, 0), "Error writing configuration to file");
            }
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
        ui.heading("KsfLogger Configuration Path");
        clickable_path(ui, logging::get_path());
        ui.separator();

        ui.columns(2, |columns| {
            columns[0].heading("Sinks");
            columns[0].horizontal_wrapped(|ui| {
                ui.label("Create New Sink:");
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

    fn loggers(&mut self, ui: &mut Ui) -> Vec<usize> {
        let mut loggers_to_remove = vec![];
        for (i, logger) in self.logger.config.loggers.iter_mut().enumerate() {
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button(" - ").on_hover_text("Remove Logger").clicked() {
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
                if ui.button(" ✅ ").on_hover_text("Enable on all loggers").clicked() {
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

fn clickable_path(ui: &mut Ui, path: PathBuf) {
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

    let icon = image::open("keysight-logo.ico").expect("Failed to open icon path").to_rgba8();
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
