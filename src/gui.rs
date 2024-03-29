use crate::gui_state::{FilterOptions, HwconfigState, IonDiagnosticsState, LoggingState, PathInfo, ReportsState, VersionsFilter, VersionsState, VersionsTypes};
use crate::logging::{Bool, Level, Logger, Sink, Template};
use crate::model::Model;
use crate::versions::{FileInfo, RequestStatus, BASE_FILE_URL};
use crate::{common, hwconfig, ion_diagnostics, logging, report, versions};
#[cfg(not(target_arch = "arm"))]
use clipboard::ClipboardProvider;
use eframe::egui::Visuals;
use eframe::{egui, egui::Ui, epi};
use std::collections::BTreeMap;
use std::io;
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
    // #[strum(serialize = "Log Viewer")]
    // LogViewer,
    Packages,
    Installers,
    Reports,
}

struct GuiApp {
    model: Box<dyn Model>,
    selected_tab: Option<Tabs>,
    hwconfig: HwconfigState,
    logger: LoggingState,
    // log_viewer: LogViewerState,
    packages: VersionsState,
    installers: VersionsState,
    reports: ReportsState,
    diagnostics: IonDiagnosticsState,

    cwd: PathBuf
}

impl epi::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
                ui.separator();
                for tab in Tabs::iter() {
                    // Hide LogViewer in Release builds for now
                    // if cfg!(debug_assertions) || tab != Tabs::LogViewer {
                    // Hide IonDiagnostics on wasm
                    // if tab != Tabs::IonDiagnostics || cfg!(not(target_arch = "wasm32")) {
                    if tab != Tabs::IonDiagnostics { // TODO: Ion Diagnostics is broken, fix it
                        self.make_tab(ui, Some(tab));
                    }
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
                    let selected_branch = self.packages.selected_branch.clone();
                    // TODO: remove model call from render loop
                    versions(ui, frame, &mut self.packages, &self.model.versions_download_dir(&selected_branch));
                }
                Some(Tabs::Installers) => {
                    let selected_branch = self.packages.selected_branch.clone();
                    // TODO: remove model call from render loop
                    versions(ui, frame, &mut self.installers, &self.model.versions_download_dir(&selected_branch));
                }
                Some(Tabs::Reports) => {
                    self.report(ui, frame);
                }
                Some(Tabs::IonDiagnostics) => {
                    self.diagnostics(ui);
                }
                // Some(Tabs::LogViewer) => {
                //     self.log_viewer(ui);
                // }
                None => {
                    about(ui);
                }
            }
        });
    }

    fn setup(&mut self, ctx: &egui::Context, _frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        ctx.set_visuals(Visuals::dark());
        self.cwd = self.model.get_cwd();

        let logging_config_path = self.model.logging_get_config_path().unwrap_or_else(|| self.in_cwd(logging::FILE_NAME));
        self.logger.config = self.model.logging_get_config_from(&logging_config_path).unwrap_or_default();
        self.logger.loaded_from = Some(logging_config_path);
        let logger_cwd_path = self.in_cwd(logging::FILE_NAME);
        self.logger.cwd_path_info = PathInfo {
            path: logger_cwd_path.clone(),
            file_exists: self.model.file_exists(&logger_cwd_path)
        };
        for path in self.model.logging_valid_paths().iter() {
            self.logger.valid_paths_info.push(PathInfo { path: path.clone(), file_exists: self.model.file_exists(path) });
        }

        let ion_debug_dir = std::env::var(ion_diagnostics::ENV_VAR).ok().map(|x| PathBuf::from(x).join(ion_diagnostics::FILE_NAME));
        let path = match ion_debug_dir.clone() {
            Some(x) if x.exists() => { x }
            _ => { self.in_cwd(ion_diagnostics::FILE_NAME) }
        };
        self.diagnostics.ion_debug_dir_info = ion_debug_dir.map(|_| PathInfo {
            path: path.clone(),
            file_exists: self.model.file_exists(&path)
        });
        self.diagnostics.config = ion_diagnostics::get_config_from(&path).unwrap_or_default();
        self.diagnostics.loaded_from = Some(path);

        let hwconfig_cwd_path = self.in_cwd(hwconfig::FILE_NAME);
        self.hwconfig.cwd_path_info = PathInfo {
            path: hwconfig_cwd_path.clone(),
            file_exists: self.model.file_exists(&hwconfig_cwd_path)
        };
        for path in hwconfig::valid_paths().iter() {
            self.hwconfig.valid_paths_info.push(PathInfo { path: path.clone(), file_exists: self.model.file_exists(path) });
        }

        self.update_report_summary();

        // let stdin_data = self.log_viewer.stdin_data.clone();
        //
        // log_viewer::watch_stdin(stdin_data, frame.clone());
    }

    fn name(&self) -> &str {
        "SigGen Toolkit"
    }
}

impl GuiApp {
    fn new(model: Box<dyn Model>) -> Self {
        Self {
            model: model,
            hwconfig: Default::default(),
            logger: Default::default(),
            // log_viewer: Default::default(),
            packages: VersionsState::new(VersionsTypes::Packages),
            installers: VersionsState::new(VersionsTypes::Installers),
            reports: Default::default(),
            diagnostics: Default::default(),
            selected_tab: Some(Tabs::LoggingConfiguration),
            cwd: Default::default(),
        }
    }

    fn make_tab(&mut self, ui: &mut Ui, tab: Option<Tabs>) {
        let text = match tab {
            None => "About".to_string(),
            Some(_) => tab.unwrap().to_string(),
        };
        if ui.selectable_label(self.selected_tab == tab, text).clicked() {
            self.selected_tab = tab;
        }
    }

    fn report(&mut self, ui: &mut Ui, frame: &epi::Frame) {
        ui.heading("Reports");
        ui.separator();

        text_edit_labeled(
            ui,
            "Name",
            &mut self.reports.name,
            Some("Descriptive name for report .zip file. Required."),
        );

        if self.reports.name_changed() {
            self.reports.zip_file_path = self.in_cwd(self.model.report_zip_file_name(&self.reports.name));
            self.reports.generate_status = None;
            self.reports.file_exists = self.model.file_exists(&self.reports.zip_file_path);
            *self.reports.upload_status.lock().unwrap() = RequestStatus::Idle;
        }

        ui.add_enabled_ui(!self.reports.name.is_empty(), |ui| {
            copyable_path(ui, &self.reports.zip_file_path);
        });

        self.report_generate_button(ui, &self.reports.zip_file_path.clone());

        ui.separator();
        ui.heading("Upload to Artifactory");
        ui.hyperlink_to(
            "Upload Location",
            format!("{}/{}", BASE_FILE_URL, versions::report_segments()),
        );
        self.report_upload_button(ui, frame, &self.reports.zip_file_path.clone());

        ui.separator();
        self.report_summary(ui);
    }

    fn report_generate_button(&mut self, ui: &mut Ui, path: &Path) {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(
                !self.reports.name.is_empty() && self.reports.generate_status != Some(true),
                |ui| {
                    if ui.button("Generate Report").clicked() {
                        self.reports.generate_status = match self.model.report_create_report(&self.reports.name) {
                            Ok(_) => Some(true),
                            Err(_) => Some(false),
                        };
                        self.reports.file_exists = self.model.file_exists(path);
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

    fn report_upload_button(&mut self, ui: &mut Ui, frame: &epi::Frame, path: &Path) {
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

                    #[cfg(not(target_arch = "arm"))]
                    if ui.button("🗐").on_hover_text(&url).clicked() {
                        if let Ok(mut clip) = clipboard::ClipboardContext::new() {
                            let _ = clip.set_contents(url);
                        }
                    }
                }
            });
        });
    }

    fn upload_clicked(&self, frame: &epi::Frame, path: &Path) {
        // TODO: backend
        if self
            .packages
            .client
            .upload_report(
                path,
                Some(self.reports.upload_status.clone()),
                Some(frame.clone()),
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
        let path = self.model.logging_get_log_path_from_current_config();
        self.reports.log_file_path = if path.exists() { Some(path) } else { None };

        let path = self.model.get_exception_log_path();
        self.reports.exception_log_file_path = if path.exists() { Some(path) } else { None };

        self.reports.log_cfg_path = self.model.logging_get_config_path();

        let path = report::get_no_reset_system_settings_path();
        self.reports.no_reset_system_settings_path = if path.exists() { Some(path) } else { None };

        self.reports.data_dir_state_files = self.model.report_get_data_dir_state_file_paths();

        self.reports.hwconfig_path = self.model.hwconfig_get_path();
        self.reports.installed_version = self.model.installed_version();

        self.reports.generate_status = None;
        *self.reports.upload_status.lock().unwrap() = RequestStatus::Idle;
    }

    fn hwconfig(&mut self, ui: &mut Ui) {
        ui.heading("Hardware Configuration");
        ui.separator();
        ui.strong("Paths indexed by SigGen:");
        for path_info in self.hwconfig.valid_paths_info.clone().iter() {
            self.hwconfig_path(ui, &path_info);
        }
        ui.strong("Current working directory:");
        self.hwconfig_path(ui, &self.hwconfig.cwd_path_info.clone());
        ui.separator();
        ui.add(egui::TextEdit::multiline(&mut self.hwconfig.text).hint_text("Enter desired hardware configuration and click Save above."));
    }

    fn hwconfig_path(&mut self, ui: &mut Ui, path_info: &PathInfo) {
        ui.horizontal(|ui| {
            copyable_path(ui, &path_info.path);
            self.hwconfig_path_buttons(ui, path_info);
        });
    }

    fn hwconfig_path_buttons(&mut self, ui: &mut Ui, path_info: &PathInfo) {
        if ui.button("Save").clicked() {
            self.hwconfig.write_error = hwconfig::set_text(&path_info.path, &self.hwconfig.text).is_err();
            self.hwconfig.remove_error = false;
        }
        if ui.add_enabled(path_info.file_exists, egui::Button::new("Delete")).clicked() {
            self.hwconfig.write_error = false;
            self.hwconfig.remove_error = self.remove_file(&path_info.path).is_err();
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
        for path in self.logger.valid_paths_info.clone().iter() {
            self.logging_path(ui, path);
        }
        ui.collapsing("Other Paths", |ui| {
            ui.strong("Current working directory:");
            self.logging_path(ui, &self.logger.cwd_path_info.clone());
        });
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.strong("Templates:");
            for template in Template::iter() {
                if ui.button(template.to_string()).clicked() {
                    self.logger.config = self.model.logging_get_template(&template);
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
                        let random_name = random_word::gen().to_string();
                        *name = random_name.clone();
                        self.logger.config.sinks.push(sink);
                        self.handle_sinks_action(SinksAction::Enable(random_name))
                    }
                }
            });

            egui::ScrollArea::vertical()
                .id_source("scroll_sinks")
                .show(&mut columns[0], |ui| {
                    if let Some(action) = self.sinks(ui) { self.handle_sinks_action(action) };
                });

            columns[1].heading("Loggers");
            columns[1].horizontal(|ui| {
                ui.label("Create new Logger:");
                if ui.button(" + ").clicked() {
                    let mut new_logger = Logger::default();
                    new_logger.sinks = self.logger.config.sinks.iter().map(|x| x.get_name().clone()).collect();
                    self.logger.config.loggers.push(new_logger);
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

    fn handle_sinks_action(&mut self, action: SinksAction) {
        match action {
            SinksAction::Remove(index) => {
                self.logger.config.sinks.remove(index);
            }
            SinksAction::Enable(sink) => {
                for logger in self.logger.config.loggers.iter_mut() {
                    logger.sinks.push(sink.clone());
                }
            }
            SinksAction::Disable(sink) => {
                for logger in self.logger.config.loggers.iter_mut() {
                    logger.sinks.retain(|s| s != &sink);
                }
            }
        }
    }


    fn logging_path(&mut self, ui: &mut Ui, path_info: &PathInfo) {
        ui.horizontal(|ui| {
            copyable_path(ui, &path_info.path);
            self.logging_path_buttons(ui, path_info);
        });
    }

    fn logging_path_buttons(&mut self, ui: &mut Ui, path_info: &PathInfo) {
        ui.label(if self.logger.loaded_from.as_ref() == Some(&path_info.path) {
            "⬅"
        } else {
            "     "
        });

        if ui.add_enabled(path_info.file_exists, egui::Button::new("Load")).clicked() {
            self.logger.config = self.model.logging_get_config_from(&path_info.path).unwrap_or_default();
            self.logger.loaded_from = Some(path_info.path.clone());
        }
        if ui.button("Save").clicked() {
            self.logger.remove_error = false;
            self.logger.write_error = self.model.logging_set_config(&path_info.path, self.logger.config.clone()).is_err();
            if !self.logger.write_error {
                self.logger.loaded_from = Some(path_info.path.clone());
            }
        }
        if ui.add_enabled(path_info.file_exists, egui::Button::new("Delete")).clicked() {
            self.logger.write_error = false;
            self.logger.remove_error = self.remove_file(&path_info.path).is_err();
            if self.logger.loaded_from == Some(path_info.path.clone()) {
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
        if self.diagnostics.ion_debug_dir_info.is_none() {
            ui.label(format!("{} environment variable must be set!", ion_diagnostics::ENV_VAR));
            ui.label("Configure the environment variable then rerun this application.");
            return
        }
        ui.strong("Paths indexed by Ion:");
        self.diagnostics_path(ui, &self.diagnostics.ion_debug_dir_info.clone().unwrap());
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
                            ui.heading((i + 1).to_string());
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
                            ui.heading((i + 1).to_string());
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

    fn diagnostics_path(&mut self, ui: &mut Ui, path_info: &PathInfo) {
        ui.horizontal(|ui| {
            copyable_path(ui, &path_info.path);
            self.diagnostics_path_buttons(ui, path_info);
        });
    }

    fn diagnostics_path_buttons(&mut self, ui: &mut Ui, path_info: &PathInfo) {
        ui.label(if self.diagnostics.loaded_from.as_ref() == Some(&path_info.path) {
            "⬅"
        } else {
            "     "
        });

        if ui.add_enabled(path_info.file_exists, egui::Button::new("Load")).clicked() {
            self.diagnostics.config = ion_diagnostics::get_config_from(&path_info.path).unwrap_or_default();
            self.diagnostics.loaded_from = Some(path_info.path.clone());
        }
        if ui.button("Save").clicked() {
            self.diagnostics.remove_error = false;
            self.diagnostics.write_error = ion_diagnostics::set_config(&path_info.path, self.diagnostics.config.clone()).is_err();
            if !self.diagnostics.write_error {
                self.diagnostics.loaded_from = Some(path_info.path.clone());
            }
        }
        if ui.add_enabled(path_info.file_exists, egui::Button::new("Delete")).clicked() {
            self.diagnostics.write_error = false;
            self.diagnostics.remove_error = self.remove_file(&path_info.path).is_err();
            if self.diagnostics.loaded_from == Some(path_info.path.clone()) {
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

    // fn log_viewer(&mut self, ui: &mut Ui) {
    //     ui.heading("Log Viewer (WIP)");
    //
    //     egui::ComboBox::from_label("Log Source")
    //         .selected_text(format!("{}", self.log_viewer.source))
    //         .show_ui(ui, |ui| {
    //             ui.selectable_value(&mut self.log_viewer.source, log_viewer::Source::Stdin, "Stdin");
    //             ui.selectable_value(&mut self.log_viewer.source, log_viewer::Source::File, "File");
    //         });
    //
    //     match self.log_viewer.source {
    //         log_viewer::Source::Stdin => {
    //         }
    //         log_viewer::Source::File => {
    //             ui.strong("Path:");
    //             ui.text_edit_singleline(&mut self.log_viewer.file_path);
    //             if ui.button("Load").clicked() {
    //                 self.log_viewer.load_file_data();
    //             }
    //         }
    //     };
    //
    //     let data = match self.log_viewer.source {
    //         log_viewer::Source::Stdin => {self.log_viewer.stdin_data.lock().unwrap()}
    //         log_viewer::Source::File => {self.log_viewer.file_data.lock().unwrap()}
    //     };
    //
    //     ui.label(data.items.len().to_string());
    //
    //     // egui::ScrollArea::vertical()
    //     //     .id_source("log_viewer scroll")
    //     //     .auto_shrink([false, false])
    //     //     .show(ui, |ui| {
    //     //         for item in data.items.iter() {
    //     //             ui.label(format!("{:?}", item));
    //     //         }
    //     //     });
    // }

    fn in_cwd<P: AsRef<Path>>(&self, file: P) -> PathBuf {
        self.cwd.join(file)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
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

fn versions(ui: &mut Ui, frame: &epi::Frame, state: &mut VersionsState, download_dir: &Path) {
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
            copyable_path(ui, download_dir);
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

fn versions_row(ui: &mut Ui, frame: &epi::Frame, state: &mut VersionsState, file_info: &FileInfo) {
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

fn download_clicked(frame: &epi::Frame, state: &mut VersionsState, file_info: &FileInfo) {
    let status = match state.which {
        VersionsTypes::Packages => &mut state.package_status,
        VersionsTypes::Installers => &mut state.installer_status,
    }
    .entry((state.selected_branch.clone(), file_info.clone()))
    .or_insert_with(|| std::sync::Arc::from(std::sync::Mutex::from(RequestStatus::Idle)));

    // TODO: backend
    if state
        .client
        .download_package(
            &state.which,
            &state.selected_branch,
            &file_info.full_name,
            status.clone(),
            frame.clone(),
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
        .selectable_label(false, path.to_str().unwrap())
        .on_hover_text("Left click to open in Explorer. Right click to copy.");

    if label.clicked() && common::open_explorer(path).is_err() {
        // Do Nothing
    }

    if label.secondary_clicked() {
        #[cfg(not(target_arch = "arm"))]
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

pub fn run(model: Box<dyn Model>) -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        println!("Model: {}", model.name());
    }

    let app = GuiApp::new(model);

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
