use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::logging::{self, LoggingConfiguration};

pub trait Model {
    fn name(&self) -> &str;
    fn file_exists(&self, path: &Path) -> bool;
    fn logging_valid_paths(&self) -> Vec<PathBuf>;
    fn logging_get_config_from(&self, path: &Path) -> Option<LoggingConfiguration>;
    fn logging_set_config(&self, path: &Path, config: LoggingConfiguration) -> anyhow::Result<()>;
}

#[derive(Default)]
pub struct NativeModel;

pub struct HttpClientModel {
    client: reqwest::blocking::Client,
}

impl Default for HttpClientModel {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(1000))
                .build()
                .expect("Unable to create web client"),
        }
    }
}

impl Model for NativeModel {
    fn name(&self) -> &str {
        "Native"
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    fn logging_valid_paths(&self) -> Vec<PathBuf> {
        logging::valid_paths()
    }

    fn logging_get_config_from(&self, path: &Path) -> Option<LoggingConfiguration> {
        logging::get_config_from(path)
    }

    fn logging_set_config(&self, path: &Path, config: LoggingConfiguration) -> anyhow::Result<()> {
        std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }
}

impl HttpClientModel {
    fn create_get_request(&self, stem: &str) -> reqwest::blocking::RequestBuilder {
        self.client.get(format!("{}/{}", "http://localhost:8000", stem)) // TODO: URL
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

    fn logging_set_config(&self, path: &Path, config: LoggingConfiguration) -> anyhow::Result<()> {
        #[cfg(debug_assertions)]
        println!("Sending logging_get_config_from request: {}", path.to_string_lossy());
        let response = self
            .client
            .post(format!(
                "{}/{}",
                "http://localhost:8000",
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
}
