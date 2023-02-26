use std::path::{Path, PathBuf};
use crate::{ion_diagnostics, logging, report};
use rocket::{
    serde::json::Json,
    get,
    post,
    launch,
    http::Status
};
use crate::ion_diagnostics::DiagnosticsConfiguration;
use crate::logging::LoggingConfiguration;

#[get("/logging/valid-paths", format = "json")]
fn get_logging_valid_paths() -> Json<Vec<PathBuf>> {
    Json(logging::valid_paths())
}

#[get("/logging/config/<path..>", format = "json")]
fn get_logging_config(path: PathBuf) -> Option<Json<LoggingConfiguration>> {
    logging::get_config_from(&Path::new("/").join(path)).map(|config| Json(config))
}

#[post("/logging/config/<path..>", format = "json", data = "<config>")]
fn set_logging_config(path: PathBuf, config: Json<LoggingConfiguration>) -> Status {
    match logging::set_config(&Path::new("/").join(path), config.into_inner()) {
        Ok(_) => {Status::Ok}
        Err(_) => {Status::InternalServerError}
    }
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

#[get("/reports/zip-file-name/<name>")]
fn get_report_zip_file_name(name: &str) -> String {
    report::zip_file_name(name)
}

#[launch]
pub fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes![
        get_logging_valid_paths,
        get_logging_config,
        set_logging_config,
        get_ion_diagnostics_config,
        set_ion_diagnostics_config,
        create_report,
        get_data_dir_state_file_paths,
        get_report_zip_file_name,
    ])
}