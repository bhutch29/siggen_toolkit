use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::{ion_diagnostics, logging, report, common, hwconfig, versions};
use rocket::{serde::json::Json, get, post, launch, http::Status, delete};
use crate::ion_diagnostics::DiagnosticsConfiguration;
use crate::logging::{LoggingConfiguration, Template};

#[get("/cwd", format = "json")]
fn get_cwd() -> Json<PathBuf> {
    Json(common::in_cwd(PathBuf::new()))
}

#[get("/logging/config-path", format = "json")]
fn get_logging_config_path() -> Json<Option<PathBuf>> {
    Json(logging::get_config_path())
}

#[get("/logging/valid-paths", format = "json")]
fn get_logging_valid_paths() -> Json<Vec<PathBuf>> {
    Json(logging::valid_paths())
}

#[get("/logging/code-path", format = "json")]
fn get_logging_code_path() -> Json<PathBuf> {
    Json(logging::get_code_defined_log_path())
}

#[get("/logging/config/<path..>", format = "json")]
fn get_logging_config(path: PathBuf) -> Option<Json<LoggingConfiguration>> {
    logging::get_config_from(&Path::new("/").join(path)).map(|config| Json(config))
}

#[get("/logging/log-path-from-current-config", format = "json")]
fn get_logging_log_path_from_current_config() -> Json<PathBuf> {
    Json(logging::get_log_path_from_current_config())
}

#[post("/logging/config/<path..>", format = "json", data = "<config>")]
fn set_logging_config(path: PathBuf, config: Json<LoggingConfiguration>) -> Status {
    match logging::set_config(&Path::new("/").join(path), config.into_inner()) {
        Ok(_) => {Status::Ok}
        Err(_) => {Status::InternalServerError}
    }
}

#[get("/logging/template/<template>", format = "json")]
fn get_logging_template(template: &str) -> Option<Json<LoggingConfiguration>> {
    Template::from_str(template).ok().map(|x| Json(logging::get_template(&x)))
}

#[get("/ion-diagnostics/config/<path..>", format = "json")]
fn get_ion_diagnostics_config(path: PathBuf) -> Option<Json<DiagnosticsConfiguration>> {
    ion_diagnostics::get_config_from(&Path::new("/").join(path)).map(|config| Json(config))
}

#[post("/ion-diagnostics/config/<path..>", data = "<config>")]
fn set_ion_diagnostics_config(path: PathBuf, config: Json<DiagnosticsConfiguration>) -> Status {
    match ion_diagnostics::set_config(&Path::new("/").join(path), config.into_inner()) {
        Ok(_) => {Status::Ok}
        Err(_) => {Status::InternalServerError}
    }
}

#[post("/reports/create/<name>")]
fn create_report(name: &str) -> Status {
    match report::create_report(name) {
        Ok(_) => {Status::Ok}
        Err(_) => {Status::InternalServerError}
    }
}

#[get("/reports/state-paths", format = "json")]
fn get_data_dir_state_file_paths() -> Json<Vec<String>> {
    Json(report::get_data_dir_state_file_paths())
}

#[get("/reports/exception-log-path", format = "json")]
fn get_exception_log_path() -> Json<PathBuf> {
    Json(logging::get_exception_log_path())
}

#[get("/reports/zip-file-name/<name>")]
fn get_report_zip_file_name(name: &str) -> String {
    report::zip_file_name(name)
}

#[get("/file-exists/<path..>", format = "json")]
fn get_file_exists(path: PathBuf) -> &'static str {
    if path.exists() { "true" } else { "false" }
}

// TODO: protections
#[delete("/delete-file/<path..>")]
fn delete_file(path: PathBuf) -> std::io::Result<()> {
    std::fs::remove_file(path)
}

#[get("/reports/exception-log-path", format = "json")]
fn get_hwconfig_path() -> Json<Option<PathBuf>> {
    Json(hwconfig::get_path())
}

#[get("/versions/installed", format = "json")]
fn get_versions_installed_version() -> Json<Option<String>> {
    Json(versions::installed_version())
}

#[get("/versions/download-dir/<branch>", format = "json")]
fn get_versions_download_dir(branch: String) -> Json<PathBuf> {
    Json(versions::download_dir(&branch))
}

#[launch]
pub fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes![
        get_cwd,
        get_logging_config_path,
        get_logging_valid_paths,
        get_logging_code_path,
        get_logging_config,
        get_logging_log_path_from_current_config,
        set_logging_config,
        get_logging_template,
        get_ion_diagnostics_config,
        set_ion_diagnostics_config,
        create_report,
        get_data_dir_state_file_paths,
        get_exception_log_path,
        get_report_zip_file_name,
        get_file_exists,
        delete_file,
        get_hwconfig_path,
        get_versions_installed_version,
        get_versions_download_dir
    ])
}
