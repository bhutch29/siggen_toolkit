use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    common, hwconfig,
    logging::{self, LoggingConfiguration, Template},
    report, versions,
};

pub trait Model {
    fn name(&self) -> &str;
    fn file_exists(&self, path: &Path) -> bool;
    fn logging_get_config_path(&self) -> Option<PathBuf>;
    fn logging_valid_paths(&self) -> Vec<PathBuf>;
    fn logging_get_config_from(&self, path: &Path) -> Option<LoggingConfiguration>;
    fn logging_get_log_path_from_current_config(&self) -> PathBuf;
    fn logging_set_config(&self, path: &Path, config: LoggingConfiguration) -> anyhow::Result<()>;
    fn logging_get_template(&self, template: &Template) -> LoggingConfiguration;
    fn get_cwd(&self) -> PathBuf;
    fn get_code_defined_log_path(&self) -> PathBuf;
    fn get_exception_log_path(&self) -> PathBuf;
    fn report_get_data_dir_state_file_paths(&self) -> Vec<String>;
    fn report_zip_file_name(&self, name: &str) -> String;
    fn report_create_report(&self, name: &str) -> anyhow::Result<()>;
    fn versions_download_dir(&self, branch: &str) -> PathBuf;
    fn hwconfig_get_path(&self) -> Option<PathBuf>;
    fn installed_version(&self) -> Option<String>;
}

#[derive(Default)]
pub struct NativeModel;

pub struct HttpClientModel {
    url: String,
    client: reqwest::blocking::Client,
}

impl Model for NativeModel {
    fn name(&self) -> &str {
        "Native"
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    fn logging_get_config_path(&self) -> Option<PathBuf> {
        logging::get_config_path()
    }

    fn logging_valid_paths(&self) -> Vec<PathBuf> {
        logging::valid_paths()
    }

    fn logging_get_config_from(&self, path: &Path) -> Option<LoggingConfiguration> {
        logging::get_config_from(path)
    }

    fn logging_get_log_path_from_current_config(&self) -> PathBuf {
        logging::get_log_path_from_current_config()
    }

    fn logging_set_config(&self, path: &Path, config: LoggingConfiguration) -> anyhow::Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    fn logging_get_template(&self, template: &Template) -> LoggingConfiguration {
        logging::get_template(template)
    }

    fn get_cwd(&self) -> PathBuf {
        common::in_cwd(PathBuf::new())
    }

    fn get_exception_log_path(&self) -> PathBuf {
        logging::get_exception_log_path()
    }

    fn get_code_defined_log_path(&self) -> PathBuf {
        logging::get_code_defined_log_path()
    }

    fn report_get_data_dir_state_file_paths(&self) -> Vec<String> {
        report::get_data_dir_state_file_paths()
    }

    fn report_zip_file_name(&self, name: &str) -> String {
        report::zip_file_name(name)
    }

    fn report_create_report(&self, name: &str) -> anyhow::Result<()> {
        report::create_report(name)
    }

    fn versions_download_dir(&self, branch: &str) -> PathBuf {
        versions::download_dir(branch)
    }

    fn hwconfig_get_path(&self) -> Option<PathBuf> {
        hwconfig::get_path()
    }

    fn installed_version(&self) -> Option<String> {
        versions::installed_version()
    }
}

impl HttpClientModel {
    pub fn new(url: String) -> Self {
        Self {
            url: url,
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(1000))
                .build()
                .expect("Unable to create web client"),
        }
    }

    fn create_get_request(&self, stem: &str) -> reqwest::blocking::RequestBuilder {
        self.client.get(format!("{}/{}", format!("http://{}", self.url), stem))
    }
}

impl Model for HttpClientModel {
    fn name(&self) -> &str {
        "Http"
    }

    fn file_exists(&self, path: &Path) -> bool {
        #[cfg(debug_assertions)]
        println!("Sending file_exists request: {}", path.to_string_lossy());
        let response = self
            .create_get_request(&format!("file-exists{}", path.to_string_lossy()))
            .send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn logging_get_config_path(&self) -> Option<PathBuf> {
        #[cfg(debug_assertions)]
        println!("Sending logging_get_path request");
        match self.create_get_request("logging/config-path").send() {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn logging_valid_paths(&self) -> Vec<PathBuf> {
        #[cfg(debug_assertions)]
        println!("Sending logging_valid_path request");
        match self.create_get_request("logging/valid-paths").send() {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn logging_get_config_from(&self, path: &Path) -> Option<LoggingConfiguration> {
        #[cfg(debug_assertions)]
        println!("Sending logging_get_config_from request: {}", path.to_string_lossy());
        let response = self
            .create_get_request(&format!("logging/config{}", path.to_string_lossy()))
            .send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn logging_get_log_path_from_current_config(&self) -> PathBuf {
        #[cfg(debug_assertions)]
        println!("Sending logging_get_log_path_from_current_config request");
        match self.create_get_request("logging/log-path-from-current-config").send() {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn logging_set_config(&self, path: &Path, config: LoggingConfiguration) -> anyhow::Result<()> {
        #[cfg(debug_assertions)]
        println!("Sending logging_set_config request: {}", path.to_string_lossy());
        let response = self
            .client
            .post(format!(
                "{}{}/{}",
                "http://",
                self.url,
                &format!("logging/config{}", path.to_string_lossy())
            ))
            .body(serde_json::to_string(&config)?)
            .send();
        match response {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("{:?}", err);
                Result::Err(err.into())
            }
        }
    }

    fn logging_get_template(&self, template: &Template) -> LoggingConfiguration {
        #[cfg(debug_assertions)]
        println!("Sending logging_get_template request: {}", template.to_string());
        let response = self
            .create_get_request(&format!("logging/template/{}", template.to_string()))
            .send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn get_cwd(&self) -> PathBuf {
        #[cfg(debug_assertions)]
        println!("Sending get_cwd request");
        let response = self.create_get_request("cwd").send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn get_exception_log_path(&self) -> PathBuf {
        #[cfg(debug_assertions)]
        println!("Sending get_exception_log_path request");
        let response = self.create_get_request("reports/exception-log-path").send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn get_code_defined_log_path(&self) -> PathBuf {
        #[cfg(debug_assertions)]
        println!("Sending get_code_defined_log_path request");
        let response = self.create_get_request("logging/code-path").send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn report_get_data_dir_state_file_paths(&self) -> Vec<String> {
        #[cfg(debug_assertions)]
        println!("Sending report_get_data_dir_state_file_paths request");
        let response = self.create_get_request("reports/state-paths").send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn report_zip_file_name(&self, name: &str) -> String {
        #[cfg(debug_assertions)]
        println!("Sending report_zip_file_name request: {}", name);
        let response = self
            .create_get_request(&format!("reports/zip-file-name/{}", name))
            .send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn report_create_report(&self, name: &str) -> anyhow::Result<()> {
        #[cfg(debug_assertions)]
        println!("Sending report_create_report request: {}", name);
        let response = self
            .client
            .post(format!(
                "{}{}/{}",
                "http://",
                self.url,
                &format!("reports/create/{}", name)
            ))
            .send();
        match response {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("{:?}", err);
                Result::Err(err.into())
            }
        }
    }

    fn versions_download_dir(&self, branch: &str) -> PathBuf {
        #[cfg(debug_assertions)]
        println!("Sending versions_download_dir request");
        let response = self
            .create_get_request(&format!("versions/download-dir/{}", branch))
            .send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn hwconfig_get_path(&self) -> Option<PathBuf> {
        #[cfg(debug_assertions)]
        println!("Sending hwconfig_get_path request");
        let response = self.create_get_request("hwconfig/path").send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }

    fn installed_version(&self) -> Option<String> {
        #[cfg(debug_assertions)]
        println!("Sending installed_version request");
        let response = self.create_get_request("versions/installed").send();
        match response {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default())
                .ok()
                .unwrap_or_default(),
            Err(err) => {
                println!("{:?}", err);
                Default::default()
            }
        }
    }
}
