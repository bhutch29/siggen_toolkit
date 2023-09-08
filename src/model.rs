use std::{path::Path, time::Duration};

pub trait Model {
    fn name(&self) -> &str;

    fn file_exists(&self, path: &Path) -> bool;
}

#[derive(Default)]
pub struct NativeModel;

pub struct HttpClientModel {
    client: reqwest::blocking::Client
}

impl Model for NativeModel {
    fn name(&self) -> &str {
        "Native"
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }
}

impl Default for HttpClientModel {
    fn default() -> Self {
        Self {
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(1000))
                .build()
                .expect("Unable to create web client")
        }
    }
}

impl Model for HttpClientModel {
    fn name(&self) -> &str {
        "Http"
    }

    fn file_exists(&self, path: &Path) -> bool {
        #[cfg(debug_assertions)] println!("Sending file_exists request: {}", path.to_string_lossy());
        let request = self.client.get(format!("{}/{}", "http://localhost:8000", format!("file-exists{}", path.to_string_lossy()))); // TODO: url
        match request.send() {
            Ok(response) => serde_json::from_str(&response.text().unwrap_or_default()).ok().unwrap_or(false),
            Err(err) => {
                println!("{:?}", err);
                false
            },
        }
    }
}
