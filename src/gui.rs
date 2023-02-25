use crate::common::in_cwd;
use crate::gui_state::{EventsState, FilterOptions, HwconfigState, IonDiagnosticsState, LoggingState, LogViewerState, ReportsState, VersionsFilter, VersionsState, VersionsTypes};
use crate::logging::{Bool, Level, Logger, Sink, Template};
use crate::versions::{FileInfo, RequestStatus, BASE_FILE_URL};
use crate::{common, events, hwconfig, ion_diagnostics, log_viewer, logging, report, versions};
use clipboard::ClipboardProvider;
use eframe::{egui, egui::Ui, epi};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use strum::{Display, EnumIter, IntoEnumIterator};
use crate::ion_diagnostics::{OperationsInstance, SettingsInstance};

enum SinksAction {
    Remove(usize),
    Enable(String),
    Disable(String),
}

#[derive(PartialEq, Clone, Copy, EnumIter, Display)]
enum Tabs {
    #[strum(serialize = "Logging Configuration")]
    LoggingConfiguration,
    #[strum(serialize = "Ion Diagnostics")]
    IonDiagnostics,
    #[strum(serialize = "Hardware Configuration")]
    HwConfig,
    #[strum(serialize = "Log Viewer")]
    LogViewer,
    Packages,
    Installers,
    Events,
    Reports,
}

struct GuiApp {
    selected_tab: Option<Tabs>,
    hwconfig: HwconfigState,
    logger: LoggingState,
    log_viewer: LogViewerState,
    packages: VersionsState,
    installers: VersionsState,
    events: EventsState,
    reports: ReportsState,
    diagnostics: IonDiagnosticsState,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            hwconfig: Default::default(),
            logger: Default::default(),
            log_viewer: Default::default(),
            packages: VersionsState::new(VersionsTypes::Packages),
            installers: VersionsState::new(VersionsTypes::Installers),
            events: Default::default(),
            reports: Default::default(),
            diagnostics: Default::default(),
            selected_tab: Some(Tabs::LoggingConfiguration),
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
                Some(Tabs::LoggingConfiguration) => {
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
                Some(Tabs::Reports) => {
                    self.report(ui, frame);
                }
                Some(Tabs::IonDiagnostics) => {
                    self.diagnostics(ui);
                }
                Some(Tabs::LogViewer) => {
                    self.log_viewer(ui);
                }
                None => {
                    about(ui);
                }
            }
        });
    }

    fn setup(&mut self, _ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) {
        self.logger.config = logging::get_config_from(&logging::get_path_or_cwd()).unwrap_or_default();
        self.logger.loaded_from = Some(logging::get_path_or_cwd());

        self.diagnostics.ion_debug_dir = std::env::var(ion_diagnostics::ENV_VAR).ok().map(|x| PathBuf::from(x).join(ion_diagnostics::FILE_NAME));
        let path = match self.diagnostics.ion_debug_dir.clone() {
            Some(x) if x.exists() => { x }
            _ => { in_cwd(ion_diagnostics::FILE_NAME) }
        };
        self.diagnostics.config = ion_diagnostics::get_config_from(&path).unwrap_or_default();
        self.diagnostics.loaded_from = Some(path);

        self.update_report_summary();

        self.events.cache = events::get_events();

        let stdin_data = self.log_viewer.stdin_data.clone();
        let repaint = frame.repaint_signal().clone();

        log_viewer::watch_stdin(stdin_data, repaint);
    }

    fn name(&self) -> &str {
        "SigGen Toolkit"
    }
}

impl GuiApp {
    fn make_tab(&mut self, ui: &mut Ui, tab: Option<Tabs>) {
        let text = match tab {
            None => "About".to_string(),
            Some(_) => tab.unwrap().to_string(),
        };
        if ui.selectable_label(self.selected_tab == tab, text).clicked() {
            self.selected_tab = tab;
        }
    }

    fn report(&mut self, ui: &mut Ui, frame: &mut epi::Frame<'_>) {
        ui.heading("Reports");
        ui.separator();

        text_edit_labeled(
            ui,
            "Name",
            &mut self.reports.name,
            Some("Descriptive name for report .zip file. Required."),
        );

        let file_name = report::zip_file_name(&self.reports.name);
        let file_path = in_cwd(&file_name);

        if self.reports.name_changed() {
            self.reports.generate_status = None;
            self.reports.file_exists = file_path.exists();
            *self.reports.upload_status.lock().unwrap() = RequestStatus::Idle;
        }

        ui.add_enabled_ui(!self.reports.name.is_empty(), |ui| {
            copyable_path(ui, &file_path);
        });

        self.report_generate_button(ui, &file_path);

        ui.separator();
        ui.heading("Upload to Artifactory");
        ui.hyperlink_to(
            "Upload Location",
            format!("{}/{}", BASE_FILE_URL, versions::report_segments()),
        );
        self.report_upload_button(ui, frame, &file_path);

        ui.separator();
        self.report_summary(ui);
    }

    fn report_generate_button(&mut self, ui: &mut Ui, path: &Path) {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(
                !self.reports.name.is_empty() && self.reports.generate_status != Some(true),
                |ui| {
                    if ui.button("Generate Report").clicked() {
                        self.reports.generate_status = match report::create_report(&self.reports.name) {
                            Ok(_) => Some(true),
                            Err(_) => Some(false),
                        };
                        self.reports.file_exists = path.exists();
                        *self.reports.upload_status.lock().unwrap() = RequestStatus::Idle;
                    }
                },
            );

            match self.reports.generate_status {
                Some(false) if !self.reports.file_exists => error_label(ui, "Generation failed"),
                Some(false) => error_label(ui, "Generation failed"),
                None if self.reports.file_exists => warning_label(ui, "File already exists, will overwrite"),
                Some(true) => {
                    ui.strong("Generation complete");
                }
                _ => {}
            }
        });
    }

    fn report_upload_button(&mut self, ui: &mut Ui, frame: &mut epi::Frame<'_>, path: &Path) {
        ui.add_enabled_ui(self.reports.file_exists, |ui| {
            ui.horizontal(|ui| match *self.reports.upload_status.lock().unwrap() {
                RequestStatus::InProgress => {
                    ui.strong("Uploading...");
                }
                RequestStatus::Error => {
                    if ui.button("⬆  Retry ").clicked() {
                        self.upload_clicked(frame, path);
                    }
                    error_label(ui, "Upload failed");
                }
                RequestStatus::Idle => {
                    if ui.button("⬆  Upload").clicked() {
                        self.upload_clicked(frame, path);
                    }
                }
                RequestStatus::Success => {
                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("⬆  Upload").clicked() {
                            // Do Nothing
                        }
                    });
                    ui.strong("Upload complete");
                    let url = format!(
                        "{}/{}/{}",
                        BASE_FILE_URL,
                        versions::report_segments(),
                        path.file_name().unwrap().to_string_lossy()
                    );
                    if ui.button("🗐").on_hover_text(&url).clicked() {
                        if let Ok(mut clip) = clipboard::ClipboardContext::new() {
                            let _ = clip.set_contents(url);
                        }
                    }
                }
            });
        });
    }

    fn upload_clicked(&self, frame: &mut epi::Frame<'_>, path: &Path) {
        if self
            .packages
            .client
            .upload_report(
                path,
                Some(self.reports.upload_status.clone()),
                Some(frame.repaint_signal().clone()),
            )
            .is_err()
        {
            *self.reports.upload_status.lock().unwrap() = RequestStatus::Error;
        }
    }

    fn report_summary(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Summary");
            if ui.button("⟳  Refresh").clicked() {
                self.update_report_summary();
            }
        });

        ui.monospace(format!(
            "Installed Version: {}",
            match &self.reports.installed_version {
                None => "Not Found",
                Some(version) => version,
            }
        ));
        ui.monospace(format!(
            "Host Name: {}",
            match &self.reports.host_name {
                None => "Not Found",
                Some(hostname) => hostname,
            }
        ));
        ui.monospace(format!(
            "Log File Path: {}",
            match &self.reports.log_file_path {
                None => "Not Found".to_string(),
                Some(path) => path.display().to_string(),
            }
        ));
        ui.monospace(format!(
            "Exception Log File Path: {}",
            match &self.reports.exception_log_file_path {
                None => "Not Found".to_string(),
                Some(path) => path.display().to_string(),
            }
        ));
        ui.monospace(format!(
            "Log Config Path: {}",
            match &self.reports.log_cfg_path {
                None => "Not Found".to_string(),
                Some(path) => path.display().to_string(),
            }
        ));
        ui.monospace(format!(
            "No Reset System Settings Path: {}",
            match &self.reports.no_reset_system_settings_path {
                None => "Not Found".to_string(),
                Some(path) => path.display().to_string(),
            }
        ));
        ui.monospace(format!(
            "Data Directory State Files: {}",
            if self.reports.data_dir_state_files.is_empty() {
                "Not Found".to_string()
            } else {
                self.reports.data_dir_state_files.join(", ")
            }
        ));
        ui.monospace(format!(
            "HW Config Path: {}",
            match &self.reports.hwconfig_path {
                None => "Not Found".to_string(),
                Some(path) => path.display().to_string(),
            }
        ));
    }

    fn update_report_summary(&mut self) {
        let path = logging::get_log_path();
        self.reports.log_file_path = if path.exists() { Some(path) } else { None };

        let path = logging::get_exception_log_path();
        self.reports.exception_log_file_path = if path.exists() { Some(path) } else { None };

        self.reports.log_cfg_path = logging::get_path();

        let path = report::get_no_reset_system_settings_path();
        self.reports.no_reset_system_settings_path = if path.exists() { Some(path) } else { None };

        self.reports.data_dir_state_files = report::get_data_dir_state_file_paths();

        self.reports.hwconfig_path = hwconfig::get_path();
        self.reports.installed_version = versions::installed_version();

        self.reports.generate_status = None;
        *self.reports.upload_status.lock().unwrap() = RequestStatus::Idle;

        self.reports.host_name = Some(gethostname::gethostname().to_string_lossy().to_string());
    }

    fn events(&mut self, ui: &mut Ui) {
        ui.heading("Events (WIP)");
        ui.separator();
        if !cfg!(windows) {
            ui.label("Event Log is only supported on Windows.");
        } else {
            egui::ScrollArea::vertical()
                .id_source("scroll_sinks")
                .show(ui, |ui| {
                    match &self.events.cache {
                        Ok(events) => {
                            for event in events {
                                ui.horizontal(|ui| {
                                    ui.label(event.system.level);
                                    ui.label(&event.system.provider.name);
                                    ui.label(&event.system.time_created.system_time);
                                });
                            }
                        }
                        Err(msg) => { ui.label(msg); }
                    };

                });


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
        self.hwconfig_path(ui, &in_cwd(hwconfig::FILE_NAME));
        ui.separator();
        ui.add(egui::TextEdit::multiline(&mut self.hwconfig.text).hint_text("Enter desired hardware configuration and click Save above."));
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
                // hwconfig::set(path, self.hwconfig.platform, self.hwconfig.channel_count, self.hwconfig.has_io_extender).is_err();
                hwconfig::set_text(path, &self.hwconfig.text).is_err();
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

    fn logging(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("KSF Logger Configuration");
            ui.checkbox(&mut self.logger.advanced, "Advanced Options");
        });
        ui.separator();
        ui.strong("Paths indexed by SigGen:");
        for path in logging::valid_paths().iter() {
            self.logging_path(ui, path);
        }
        ui.strong("Current working directory:");
        self.logging_path(ui, &in_cwd(logging::FILE_NAME));
        ui.strong("Custom:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.logger.custom_path);
            self.logging_path_buttons(ui, PathBuf::from(&self.logger.custom_path));
        });
        ui.separator();

        ui.strong("Templates");
        ui.horizontal_wrapped(|ui| {
            for template in Template::iter() {
                if ui.button(template.to_string()).clicked() {
                    self.logger.config = logging::get_template(&template);
                    self.logger.loaded_from = None;
                }
            }
        });
        ui.separator();

        ui.columns(2, |columns| {
            columns[0].heading("Sinks");
            columns[0].horizontal_wrapped(|ui| {
                ui.label("Create new Sink:");
                for mut sink in Sink::iter() {
                    let sink_name = sink.to_string();
                    if ui.button(sink_name.clone()).clicked() {
                        let (name, _level) = sink.get_name_and_level_as_mut();
                        *name = random_word::gen().to_string();
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
            self.logger.write_error = logging::set_config(&path, self.logger.config.clone()).is_err();
            if !self.logger.write_error {
                self.logger.loaded_from = Some(path.clone());
            }
        }
        if ui.add_enabled(exists, egui::Button::new("Delete")).clicked() {
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
                ui.add(egui::TextEdit::singleline(&mut logger.name).hint_text("Pattern to match").desired_width(150.0));
                level_dropdown(ui, &mut logger.level, format!("{} {}", &logger.name, i));
            });
            if self.logger.advanced {
                sinks_checkboxes(ui, logger, &self.logger.config.sinks);
            }
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

                if self.logger.advanced {
                    if ui.button(" ➕ ").on_hover_text("Enable on all loggers").clicked() {
                        action = Some(SinksAction::Enable(sink.get_name().clone()));
                    }
                    if ui.button(" ➖ ").on_hover_text("Disable on all loggers").clicked() {
                        action = Some(SinksAction::Disable(sink.get_name().clone()));
                    }
                }
            });

            let (name, level) = sink.get_name_and_level_as_mut();
            if self.logger.advanced {
                text_edit_labeled(ui, "Name", name, Some("Unique name required"));
            }
            level_dropdown(ui, level, format!("{} {}", name, i));

            match sink {
                Sink::RotatingFile {
                    ref mut file_name,
                    ref mut truncate,
                    ref mut max_size,
                    ref mut max_files,
                    ..
                } => {
                    text_edit_labeled(ui, "File Path", file_name, None);
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
                    text_edit_labeled(ui, "File Path", file_name, None);
                    truncate_ui(ui, truncate);
                }
                Sink::DailyFile {
                    ref mut file_name,
                    ref mut truncate,
                    ..
                } => {
                    text_edit_labeled(ui, "File Path", file_name, None);
                    truncate_ui(ui, truncate);
                }
                Sink::Console { ref mut is_color, .. } => {
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
                    text_edit_labeled(ui, "Url", url, None);
                }
            }
        }
        action
    }

    fn diagnostics(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Ion Diagnostics");
            ui.hyperlink_to("Confluence Page", ion_diagnostics::CONFLUENCE_URL);
        });
        ui.separator();
        if self.diagnostics.ion_debug_dir.is_none() {
            ui.label(format!("{} environment variable must be set!", ion_diagnostics::ENV_VAR));
            ui.label("Configure the environment variable then rerun this application.");
            return
        }
        ui.strong("Paths indexed by Ion:");
        self.diagnostics_path(ui, &self.diagnostics.ion_debug_dir.clone().unwrap());
        ui.strong("Current working directory:");
        self.diagnostics_path(ui, &in_cwd(ion_diagnostics::FILE_NAME));
        ui.strong("Custom:");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.diagnostics.custom_path);
            self.diagnostics_path_buttons(ui, PathBuf::from(&self.diagnostics.custom_path));
        });
        ui.separator();

        ui.columns(2, |columns| {
            columns[0].heading("Settings");
            columns[0].separator();

            egui::ScrollArea::vertical()
                .id_source("scroll_settings")
                .show(&mut columns[0], |ui| {
                    ui.heading("Global");

                    ui.checkbox(&mut self.diagnostics.config.settings.global.all_enabled, "allEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_resolve_enabled, "settingResolveEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_set_enabled, "settingSetEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_set_by_user_enabled, "settingSetByUserEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_marked_enabled, "settingMarkedEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_op_value_updated, "settingOpValueUpdated");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.this_setting_resolve_enabled, "thisSettingResolveEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.this_setting_set_enabled, "thisSettingSetEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.this_setting_set_by_user_enabled, "thisSettingSetByUserEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.this_setting_marked_enabled, "thisSettingMarkedEnabled");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.this_setting_op_value_updated, "thisSettingOpValueUpdated");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_registered_with_gui, "settingRegisteredWithGui");
                    ui.checkbox(&mut self.diagnostics.config.settings.global.setting_log_control_seos, "settingLogControlSeos");

                    ui.separator();
                    ui.heading("Instances");
                    ui.horizontal(|ui| {
                        ui.label("Create new Setting Instance:");
                        if ui.button(" + ").clicked() {
                            self.diagnostics.config.settings.instance.push(SettingsInstance::default());
                        }
                    });

                    let mut instances_to_remove = Vec::new();

                    for (i, instance) in self.diagnostics.config.settings.instance.iter_mut().enumerate() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button(" x ").on_hover_text("Remove").clicked() {
                                instances_to_remove.push(i);
                            }
                            ui.heading(i + 1);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Create new Setting Path:");
                            if ui.button(" + ").clicked() {
                                instance.setting_paths.push(String::default());
                            }
                        });

                        let mut paths_to_remove = Vec::new();

                        for (j, path) in instance.setting_paths.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(path).hint_text("Pattern to match"));
                                if ui.button(" x ").on_hover_text("Remove").clicked() {
                                    paths_to_remove.push(j);
                                }
                            });

                        }

                        for index in paths_to_remove {
                            instance.setting_paths.remove(index);
                        }

                        ui.checkbox(&mut instance.flags.trace_enabled, "traceEnabled");
                        ui.checkbox(&mut instance.flags.break_on_set, "breakOnSet");
                        ui.add(egui::Slider::new(&mut instance.flags.break_on_set_after_n, -1..=10).text("breakOnSetAfterN"));
                        ui.checkbox(&mut instance.flags.break_on_set_by_user, "breakOnSetByUser");
                        ui.add(egui::Slider::new(&mut instance.flags.break_on_set_by_user_after_n, -1..=10).text("breakOnSetByUserAfterN"));
                        ui.checkbox(&mut instance.flags.break_on_marked, "breakOnMarked");
                        ui.add(egui::Slider::new(&mut instance.flags.break_on_marked_after_n, -1..=10).text("breakOnMarkedAfterN"));
                        ui.checkbox(&mut instance.flags.break_on_resolve, "breakOnResolve");
                        ui.add(egui::Slider::new(&mut instance.flags.break_on_resolve_after_n, -1..=10).text("breakOnResolvedAfterN"));
                    }

                    for index in instances_to_remove {
                       self.diagnostics.config.settings.instance.remove(index);
                    }

                });

            columns[1].heading("Operations");
            columns[1].separator();

            egui::ScrollArea::vertical()
                .id_source("scroll_operations")
                .show(&mut columns[1], |ui| {
                    ui.heading("Global");

                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_all, "traceAll");
                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_on_mark, "traceOnMark");
                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_on_resolve, "traceOnResolve");
                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_on_abort, "traceOnAbort");
                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_on_remove, "traceOnRemove");
                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_on_add, "traceOnAdd");
                    ui.checkbox(&mut self.diagnostics.config.operations.global.trace_on_bind, "traceOnBind");

                    ui.separator();
                    ui.heading("Instances");
                    ui.horizontal(|ui| {
                        ui.label("Create new Operations Instance:");
                        if ui.button(" + ").clicked() {
                            self.diagnostics.config.operations.instance.push(OperationsInstance::default());
                        }
                    });

                    let mut instances_to_remove = Vec::new();

                    for (i, instance) in self.diagnostics.config.operations.instance.iter_mut().enumerate() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button(" x ").on_hover_text("Remove").clicked() {
                                instances_to_remove.push(i);
                            }
                            ui.heading(i + 1);
                        });

                        ui.horizontal(|ui| {
                            ui.label("Create new Operation Name:");
                            if ui.button(" + ").clicked() {
                                instance.names.push(String::default());
                            }
                        });

                        let mut paths_to_remove = Vec::new();

                        for (j, path) in instance.names.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(path).hint_text("Operation name"));
                                if ui.button(" x ").on_hover_text("Remove").clicked() {
                                    paths_to_remove.push(j);
                                }
                            });
                        }

                        ui.checkbox(&mut instance.flags.trace_all, "traceAll");
                        ui.checkbox(&mut instance.flags.trace_on_mark, "traceOnMark");
                        ui.checkbox(&mut instance.flags.trace_on_resolve, "traceOnResolve");
                        ui.checkbox(&mut instance.flags.trace_on_abort, "traceOnAbort");
                        ui.checkbox(&mut instance.flags.trace_on_remove, "traceOnRemove");
                        ui.checkbox(&mut instance.flags.trace_on_add, "traceOnAdd");
                        ui.checkbox(&mut instance.flags.trace_on_bind, "traceOnBind");
                        ui.checkbox(&mut instance.flags.break_on_mark, "breakOnMark");
                        ui.add(egui::Slider::new(&mut instance.flags.break_on_mark_after_n, -1..=10).text("breakOnMarkAfterN"));
                        ui.checkbox(&mut instance.flags.break_on_abort, "breakOnAbort");
                    }

                    for index in instances_to_remove {
                        self.diagnostics.config.operations.instance.remove(index);
                    }
                });
        });
    }

    fn diagnostics_path(&mut self, ui: &mut Ui, path: &Path) {
        ui.horizontal(|ui| {
            copyable_path(ui, path);
            self.diagnostics_path_buttons(ui, PathBuf::from(path));
        });
    }

    fn diagnostics_path_buttons(&mut self, ui: &mut Ui, path: PathBuf) {
        ui.label(if self.diagnostics.loaded_from.as_ref() == Some(&path) {
            "⬅"
        } else {
            "     "
        });

        let exists = path.exists() && path.is_file();
        if ui.add_enabled(exists, egui::Button::new("Load")).clicked() {
            self.diagnostics.config = ion_diagnostics::get_config_from(&path).unwrap_or_default();
            self.diagnostics.loaded_from = Some(path.clone());
        }
        if ui.button("Save").clicked() {
            self.diagnostics.remove_error = false;
            self.diagnostics.write_error = ion_diagnostics::set_config(&path, self.diagnostics.config.clone()).is_err();
            if !self.diagnostics.write_error {
                self.diagnostics.loaded_from = Some(path.clone());
            }
        }
        if ui.add_enabled(exists, egui::Button::new("Delete")).clicked() {
            self.diagnostics.write_error = false;
            self.diagnostics.remove_error = std::fs::remove_file(&path).is_err();
            if self.diagnostics.loaded_from == Some(path) {
                self.diagnostics.loaded_from = None;
            }
        }

        if self.diagnostics.write_error {
            error_label(ui, "Error writing configuration to file");
        }
        if self.diagnostics.remove_error {
            error_label(ui, "Error removing file");
        }
    }

    fn log_viewer(&mut self, ui: &mut Ui) {
        ui.heading("Log Viewer (WIP)");

        egui::ComboBox::from_label("Log Source")
            .selected_text(format!("{}", self.log_viewer.source))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.log_viewer.source, log_viewer::Source::Stdin, "Stdin");
                ui.selectable_value(&mut self.log_viewer.source, log_viewer::Source::File, "File");
            });

        match self.log_viewer.source {
            log_viewer::Source::Stdin => {
            }
            log_viewer::Source::File => {
                ui.strong("Path:");
                ui.text_edit_singleline(&mut self.log_viewer.file_path);
                if ui.button("Load").clicked() {
                    self.log_viewer.load_file_data();
                }
            }
        };

        let data = match self.log_viewer.source {
            log_viewer::Source::Stdin => {self.log_viewer.stdin_data.lock().unwrap()}
            log_viewer::Source::File => {self.log_viewer.file_data.lock().unwrap()}
        };

        ui.label(data.items.len());

        // egui::ScrollArea::vertical()
        //     .id_source("log_viewer scroll")
        //     .auto_shrink([false, false])
        //     .show(ui, |ui| {
        //         for item in data.items.iter() {
        //             ui.label(format!("{:?}", item));
        //         }
        //     });
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
        columns[0].horizontal(|ui| {
            ui.label("Download Directory:");
            copyable_path(ui, &versions::download_dir(&state.selected_branch));
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
    let major_copy = filter.major_filter;
    filter_dropdown(ui, "Major", &mut filter.major_filter, options);
    if filter.major_filter != major_copy {
        filter.minor_filter = None;
        filter.patch_filter = None;
    }
    if let Some(major_filter) = filter.major_filter {
        let options = &options.get(&major_filter).unwrap().next;
        let minor_copy = filter.minor_filter;
        filter_dropdown(ui, "Minor", &mut filter.minor_filter, options);
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

fn versions_row(ui: &mut Ui, frame: &mut epi::Frame<'_>, state: &mut VersionsState, file_info: &FileInfo) {
    ui.horizontal(|ui| {
        ui.monospace(format!(
            "{:12} {:15}",
            file_info.version,
            format!("({})", file_info.date)
        ));
        match state.get_download_status(file_info) {
            RequestStatus::InProgress => {
                ui.strong("Downloading...");
            }
            RequestStatus::Error => {
                if ui.button("⬇  Retry ").clicked() {
                    download_clicked(frame, state, file_info);
                }
                error_label(ui, "Download failed");
            }
            RequestStatus::Idle => {
                if ui.button("⬇  Download").clicked() {
                    download_clicked(frame, state, file_info);
                }
            }
            RequestStatus::Success => {
                ui.strong("Download complete");
            }
        }
    })
    .response
    .on_hover_text(&file_info.full_name);
}

fn download_clicked(frame: &mut epi::Frame<'_>, state: &mut VersionsState, file_info: &FileInfo) {
    let status = match state.which {
        VersionsTypes::Packages => &mut state.package_status,
        VersionsTypes::Installers => &mut state.installer_status,
    }
    .entry((state.selected_branch.clone(), file_info.clone()))
    .or_insert_with(|| std::sync::Arc::from(std::sync::Mutex::from(RequestStatus::Idle)));

    if state
        .client
        .download_package(
            &state.which,
            &state.selected_branch,
            &file_info.full_name,
            status.clone(),
            frame.repaint_signal().clone(),
        )
        .is_err()
    {
        *status.lock().unwrap() = RequestStatus::Error;
    }
}

fn text_edit_labeled(ui: &mut Ui, label: &str, content: &mut String, hint_text: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut editor = egui::TextEdit::singleline(content);
        if let Some(hint_text) = hint_text {
            editor = editor.hint_text(hint_text);
        }
        ui.add(editor);
    });
}

fn error_label(ui: &mut Ui, label: &str) {
    ui.colored_label(egui::Color32::from_rgb(255, 0, 0), label);
}

fn warning_label(ui: &mut Ui, label: &str) {
    ui.colored_label(egui::Color32::from_rgb(255, 255, 0), format!("⚠ {}", label));
}

fn copyable_path(ui: &mut Ui, path: &Path) {
    let label = ui
        .selectable_label(false, &path.to_string_lossy())
        .on_hover_text("Left click to open in Explorer. Right click to copy.");

    if label.clicked() && common::open_explorer(path).is_err() {
        // Do Nothing
    }

    if label.secondary_clicked() {
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

fn sinks_checkboxes(ui: &mut Ui, logger: &mut Logger, sinks: &[Sink]) {
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
            .selected_text(if filter_val.is_none() {
                "*".to_string()
            } else {
                filter_val.unwrap().to_string()
            })
            .show_ui(ui, |ui| {
                if !only_one_key {
                    ui.selectable_value(filter_val, None, "*");
                }
                for key in filter_options.keys() {
                    ui.selectable_value(filter_val, Some(*key), key.to_string());
                }
            });
    });
}

pub fn run() -> anyhow::Result<()> {
    let app = GuiApp::default();

    let icon_bytes = include_bytes!("../keysight-logo-gear.ico");
    let options = match image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Ico) {
        Ok(icon) => {
            let icon = icon.to_rgba8();
            let (icon_width, icon_height) = icon.dimensions();

            eframe::NativeOptions {
                icon_data: Some(epi::IconData {
                    rgba: icon.into_raw(),
                    width: icon_width,
                    height: icon_height,
                }),
                initial_window_size: Some(egui::Vec2::new(800.0, 800.0)),
                ..Default::default()
            }
        }
        Err(_) => eframe::NativeOptions::default(),
    };

    eframe::run_native(Box::new(app), options);
}
