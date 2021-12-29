use crate::cli::SimulatedChannel;
use crate::gui_state::{
    FilterOptions, HwconfigState, LoggingState, VersionsFilter, VersionsState, VersionsTypes,
};
use crate::logging::{Bool, Level, Logger, Sink};
use crate::versions::{DownloadStatus, FileInfo};
use crate::{common, hwconfig, logging};
use clipboard::ClipboardProvider;
use eframe::{egui, egui::Ui, epi};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use strum::{Display, EnumIter, IntoEnumIterator};

enum SinksAction {
    Remove(usize),
    Enable(String),
    Disable(String),
}

#[derive(PartialEq, Clone, Copy, EnumIter, Display)]
enum Tabs {
    Logging,
    #[strum(serialize = "Hardware Configuration")]
    HwConfig,
    Packages,
    Installers,
    Events,
    Report,
}

struct GuiApp {
    selected_tab: Option<Tabs>,
    hwconfig: HwconfigState,
    logger: LoggingState,
    packages: VersionsState,
    installers: VersionsState,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            hwconfig: Default::default(),
            logger: Default::default(),
            packages: VersionsState::new(VersionsTypes::Packages),
            installers: VersionsState::new(VersionsTypes::Installers),
            selected_tab: Some(Tabs::Logging),
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
                    self.make_tab(ui, Some(tab));
                }

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    self.make_tab(ui, None);
                    ui.separator();
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing = egui::style::Spacing {
                item_spacing: egui::Vec2::new(8.0, 5.0),
                scroll_bar_width: 10.0,
                button_padding: egui::Vec2::new(4.0, 4.0),
                interact_size: egui::Vec2::new(40.0, 24.0), // Needs to adjust based on button_padding

                ..egui::style::Spacing::default()
            };

            match self.selected_tab {
                Some(Tabs::HwConfig) => {
                    self.hwconfig(ui);
                }
                Some(Tabs::Logging) => {
                    self.logging(ui);
                }
                Some(Tabs::Packages) => {
                    versions(ui, frame, &mut self.packages);
                }
                Some(Tabs::Installers) => {
                    versions(ui, frame, &mut self.installers);
                }
                Some(Tabs::Events) => {
                    self.events(ui);
                }
                Some(Tabs::Report) => {
                    // TODO
                }
                None => {
                    about(ui);
                }
            }
        });
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.logger.config =
            logging::get_config_from(&logging::get_path_or_cwd()).unwrap_or_default();
        self.logger.loaded_from = Some(logging::get_path_or_cwd());
    }

    fn name(&self) -> &str {
        "SigGen Toolkit"
    }
}

impl GuiApp {
    fn make_tab(&mut self, ui: &mut Ui, tab: Option<Tabs>) {
        if ui
            .selectable_label(
                self.selected_tab == tab,
                if tab.is_none() {
                    "About".to_string()
                } else {
                    tab.unwrap().to_string()
                },
            )
            .clicked()
        {
            self.selected_tab = tab;
        }
    }

    fn events(&mut self, ui: &mut Ui) {
        ui.heading("Events");
        ui.separator();
        if !cfg!(windows) {
            ui.label("Event Log is only supported on Windows.");
        } else {
            // TODO
        }
    }

    fn hwconfig(&mut self, ui: &mut Ui) {
        ui.heading("Hardware Configuration");
        ui.separator();
        ui.strong("Paths indexed by SigGen:");
        for path in hwconfig::valid_paths().iter() {
            self.hwconfig_path(ui, path);
        }
        ui.strong("Current working directory:");
        self.hwconfig_path(ui, &common::in_cwd(hwconfig::FILE_NAME));
        ui.separator();

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

    fn hwconfig_path(&mut self, ui: &mut Ui, path: &Path) {
        ui.horizontal(|ui| {
            copyable_path(ui, path);
            self.hwconfig_path_buttons(ui, path);
        });
    }

    fn hwconfig_path_buttons(&mut self, ui: &mut Ui, path: &Path) {
        if ui.button("Save").clicked() {
            self.hwconfig.write_error =
                hwconfig::set(path, self.hwconfig.platform, self.hwconfig.channel_count).is_err();
            self.hwconfig.remove_error = false;
        }
        if ui
            .add_enabled(path.exists() && path.is_file(), egui::Button::new("Delete"))
            .clicked()
        {
            self.hwconfig.write_error = false;
            self.hwconfig.remove_error = std::fs::remove_file(path).is_err();
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
        ui.separator();
        ui.strong("Paths indexed by SigGen:");
        for path in logging::valid_paths().iter() {
            self.logging_path(ui, path);
        }
        ui.strong("Current working directory:");
        self.logging_path(ui, &common::in_cwd(logging::FILE_NAME));
        ui.strong("Custom:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.logger.custom_path);
            self.logging_path_buttons(ui, PathBuf::from(&self.logger.custom_path));
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
                    let action = self.sinks(ui);

                    match action {
                        None => {}
                        Some(SinksAction::Remove(index)) => {
                            self.logger.config.sinks.remove(index);
                        }
                        Some(SinksAction::Enable(sink)) => {
                            for logger in self.logger.config.loggers.iter_mut() {
                                logger.sinks.push(sink.clone());
                            }
                        }
                        Some(SinksAction::Disable(sink)) => {
                            for logger in self.logger.config.loggers.iter_mut() {
                                logger.sinks.retain(|s| s != &sink);
                            }
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

    fn logging_path(&mut self, ui: &mut Ui, path: &Path) {
        ui.horizontal(|ui| {
            copyable_path(ui, path);
            self.logging_path_buttons(ui, PathBuf::from(path));
        });
    }

    fn logging_path_buttons(&mut self, ui: &mut Ui, path: PathBuf) {
        ui.label(if self.logger.loaded_from.as_ref() == Some(&path) {
            "⬅"
        } else {
            "     "
        });

        let exists = path.exists() && path.is_file();
        if ui.add_enabled(exists, egui::Button::new("Load")).clicked() {
            self.logger.config = logging::get_config_from(&path).unwrap_or_default();
            self.logger.loaded_from = Some(path.clone());
        }
        if ui.button("Save").clicked() {
            self.logger.remove_error = false;
            self.logger.write_error =
                logging::set_config(&path, self.logger.config.clone()).is_err();
            if !self.logger.write_error {
                self.logger.loaded_from = Some(path.clone());
            }
        }
        if ui
            .add_enabled(exists, egui::Button::new("Delete"))
            .clicked()
        {
            self.logger.write_error = false;
            self.logger.remove_error = std::fs::remove_file(&path).is_err();
            if self.logger.loaded_from == Some(path) {
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

    fn sinks(&mut self, ui: &mut Ui) -> Option<SinksAction> {
        let mut action = None;
        for (i, sink) in self.logger.config.sinks.iter_mut().enumerate() {
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(" 🗙 ").on_hover_text("Remove").clicked() {
                    action = Some(SinksAction::Remove(i));
                }
                ui.strong(sink.to_string());
                if ui
                    .button(" ➕ ")
                    .on_hover_text("Enable on all loggers")
                    .clicked()
                {
                    action = Some(SinksAction::Enable(sink.get_name().clone()));
                }
                if ui
                    .button(" ➖ ")
                    .on_hover_text("Disable on all loggers")
                    .clicked()
                {
                    action = Some(SinksAction::Disable(sink.get_name().clone()));
                }
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
        action
    }
}

fn about(ui: &mut Ui) {
    ui.heading("About This Tool");
    ui.separator();
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
    ui.label(format!("Version: {}", VERSION));
    ui.label(format!("Authors: {}", AUTHORS));
}

fn versions(ui: &mut Ui, frame: &mut epi::Frame<'_>, state: &mut VersionsState) {
    state.setup_if_needed();

    ui.horizontal(|ui| {
        ui.heading(match state.which {
            VersionsTypes::Packages => "SigGen Packages",
            VersionsTypes::Installers => "SigGen Installers",
        });

        if ui.button("⟳  Refresh").clicked() {
            state.refresh();
        }
    });

    ui.separator();

    ui.columns(2, |columns| {
        columns[0].strong("Branch:");
        egui::ComboBox::from_id_source("branches_dropdown")
            .selected_text(&state.selected_branch)
            .show_ui(&mut columns[0], |ui| {
                for branch in &state.branch_names {
                    ui.selectable_value(&mut state.selected_branch, branch.clone(), branch);
                }
            });
        columns[0].separator();

        if let Some(latest) = state.latest() {
            columns[0].strong("Latest Version: ");
            versions_row(&mut columns[0], frame, state, &latest);
            columns[0].separator();
        }

        if let Some(filter) = state.get_current_filter_mut() {
            columns[0].strong("Version Filters:");
            version_filters(&mut columns[0], filter)
        }

        columns[1].with_layout(egui::Layout::left_to_right(), |ui| {
            ui.separator();
            ui.with_layout(egui::Layout::default(), |ui| {
                ui.heading("Versions");
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_source("versions_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        state.update_current_cache_if_needed();
                        for info in state.get_current_cache().unwrap().clone() {
                            if !state.filter_match(&info.version) {
                                continue;
                            }
                            versions_row(ui, frame, state, &info);
                            ui.separator();
                        }
                    });
            });
        });
    });
}

fn version_filters(ui: &mut Ui, filter: &mut VersionsFilter) {
    let options = &filter.options.next;
    let major_copy = filter.major_filter.clone();
    filter_dropdown(ui, "Major", &mut filter.major_filter, options);
    if filter.major_filter != major_copy {
        filter.minor_filter = None;
        filter.patch_filter = None;
    }
    if let Some(major_filter) = filter.major_filter {
        let options = &options.get(&major_filter).unwrap().next;
        let minor_copy = filter.minor_filter.clone();
        filter_dropdown(ui, "Minor", &mut filter.minor_filter, &options);
        if filter.minor_filter != minor_copy {
            filter.patch_filter = None;
        }
        if let Some(minor_filter) = filter.minor_filter {
            let options = &options.get(&minor_filter).unwrap().next;
            filter_dropdown(ui, "Patch", &mut filter.patch_filter, options);
        }
    }

    match (filter.major_filter, filter.minor_filter) {
        (None, _) => {
            filter.minor_filter = None;
            filter.patch_filter = None;
        }
        (_, None) => {
            filter.patch_filter = None;
        }
        _ => {}
    }
}

fn versions_row(
    ui: &mut Ui,
    frame: &mut epi::Frame<'_>,
    state: &mut VersionsState,
    file_info: &FileInfo,
) {
    ui.horizontal(|ui| {
        ui.monospace(format!(
            "{:12} {:15}",
            file_info.version,
            format!("({})", file_info.date)
        ));
        match state.get_package_download_status(file_info) {
            DownloadStatus::Downloading => {
                ui.strong("Downloading...");
            }
            DownloadStatus::Error => {
                error_label(ui, "Download Failed");
                if ui.button("⬇  Retry ").clicked() {
                    download_clicked(frame, state, &file_info);
                }
            }
            _ => {
                if ui.button("⬇  Download").clicked() {
                    download_clicked(frame, state, &file_info);
                }
            }
        }
    })
    .response
    .on_hover_text(&file_info.full_name);
}

fn download_clicked(frame: &mut epi::Frame<'_>, state: &mut VersionsState, file_info: &FileInfo) {
    let status = state
        .status
        .entry((state.selected_branch.clone(), file_info.clone()))
        .or_insert(std::sync::Arc::from(std::sync::Mutex::from(
            DownloadStatus::Idle,
        )));

    if let Err(_) = state.client.download_package(
        &state.selected_branch,
        &file_info.full_name,
        status.clone(),
        frame.repaint_signal().clone(),
    ) {
        let mut status_lock = status.lock().unwrap();
        *status_lock = DownloadStatus::Error;
    }
}

fn text_edit_labeled(ui: &mut Ui, label: &str, file_name: &mut String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.text_edit_singleline(file_name);
    });
}

fn error_label(ui: &mut Ui, label: &str) {
    ui.colored_label(egui::Color32::from_rgb(255, 0, 0), label);
}

fn copyable_path(ui: &mut Ui, path: &Path) {
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

    logging::remove_invalid_sinks(logger, sinks);
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

fn filter_dropdown(
    ui: &mut Ui,
    label: &str,
    filter_val: &mut Option<u16>,
    filter_options: &BTreeMap<u16, FilterOptions>,
) {
    let only_one_key = filter_options.keys().len() == 1;
    if only_one_key {
        *filter_val = filter_options.keys().next().cloned();
    }
    ui.add_enabled_ui(!only_one_key, |ui| {
        egui::ComboBox::from_label(label)
            .selected_text(format!(
                "{}",
                if filter_val.is_none() {
                    "*".to_string()
                } else {
                    filter_val.unwrap().to_string()
                }
            ))
            .show_ui(ui, |ui| {
                if !only_one_key {
                    ui.selectable_value(filter_val, None, "*");
                }
                for (key, _) in filter_options {
                    ui.selectable_value(filter_val, Some(*key), key.to_string());
                }
            });
    });
}

pub fn run() -> anyhow::Result<()> {
    let app = GuiApp::default();

    let icon_bytes = include_bytes!("../keysight-logo.ico");
    let options = match image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Ico) {
        Ok(icon) => {
            let icon = icon.to_rgba8();
            let (icon_width, icon_height) = icon.dimensions();

            eframe::NativeOptions {
                icon_data: Some(eframe::epi::IconData {
                    rgba: icon.into_raw(),
                    width: icon_width,
                    height: icon_height,
                }),
                initial_window_size: Some(egui::Vec2::new(700.0, 700.0)),
                ..Default::default()
            }
        }
        Err(_) => eframe::NativeOptions::default(),
    };

    eframe::run_native(Box::new(app), options);
}
