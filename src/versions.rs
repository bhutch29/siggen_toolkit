use anyhow::Result;
use eframe::epi::RepaintSignal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct VersionsClient {
    client: Arc<reqwest::blocking::Client>,
}

impl Default for VersionsClient {
    fn default() -> Self {
        Self {
            client: Arc::from(
                reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(1000))
                    .build()
                    .expect("Unable to create web client"),
            ),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactoryDirectory {
    pub repo: String,
    pub path: String,
    pub created: String,
    pub created_by: String,
    pub last_modified: String,
    pub modified_by: String,
    pub last_updated: String,
    pub children: Vec<ArtifactoryDirectoryChild>,
    pub uri: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactoryDirectoryChild {
    pub uri: String,
    pub folder: bool,
}

#[derive(PartialEq, Clone)]
pub enum DownloadStatus {
    Idle,
    Downloading,
    Error
}

#[derive(Default, PartialEq)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub prerelease: Option<u16>,
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &SemVer) -> Option<Ordering> {
        if self.major != other.major {
            return Some(self.major.cmp(&other.major));
        }
        if self.minor != other.minor {
            return Some(self.minor.cmp(&other.minor));
        }
        if self.patch != other.patch {
            return Some(self.patch.cmp(&other.patch));
        }
        if self.prerelease.is_some() && other.prerelease.is_some() {
            return Some(self.prerelease.cmp(&other.prerelease));
        }
        if self.prerelease.is_some() {
            return Some(Ordering::Less);
        }
        if other.prerelease.is_some() {
            return Some(Ordering::Greater);
        }
        Some(Ordering::Equal)
    }
}

pub fn parse_semver(semver: &str) -> Option<SemVer> {
    let parts: Vec<Option<u16>> = semver
        .split("-")
        .map(|x| x.parse::<u16>().ok())
        .take(4)
        .collect();

    match parts[..] {
        [Some(major), Some(minor), Some(patch), ..] => {
            let mut result = SemVer {
                major,
                minor,
                patch,
                prerelease: None,
            };

            if parts.len() == 4 {
                result.prerelease = parts[3];
            }

            Some(result)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::versions;
    use crate::versions::parse_semver;
    use std::cmp::Ordering;

    fn make_semver(
        major: u16,
        minor: u16,
        patch: u16,
        prerelease: Option<u16>,
    ) -> versions::SemVer {
        versions::SemVer {
            major,
            minor,
            patch,
            prerelease,
        }
    }

    #[test]
    fn parse() {
        assert_eq!(parse_semver("1-2-3-4").unwrap().prerelease, Some(4));
        assert_eq!(parse_semver("1-2-3").unwrap().prerelease, None);
        assert_eq!(parse_semver("1-2-3-4-5").unwrap().prerelease, Some(4));
        assert_eq!(parse_semver("1-2").is_none(), true);
        assert_eq!(parse_semver("1-2-l").is_none(), true);
    }

    #[test]
    fn major() {
        let a = make_semver(1, 2, 3, None);
        let b = make_semver(2, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn minor() {
        let a = make_semver(1, 3, 3, None);
        let b = make_semver(1, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
    }

    #[test]
    fn patch() {
        let a = make_semver(1, 2, 3, None);
        let b = make_semver(1, 2, 4, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
    }

    #[test]
    fn same() {
        let a = make_semver(1, 2, 3, None);
        let b = make_semver(1, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
    }

    #[test]
    fn prerelease() {
        let mut a = make_semver(1, 2, 3, Some(9));
        let mut b = make_semver(1, 2, 3, None);
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        a = make_semver(1, 2, 3, Some(8));
        b = make_semver(1, 2, 3, Some(9));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
        a = make_semver(1, 2, 3, Some(9));
        b = make_semver(1, 2, 3, Some(9));
        assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
    }
}

impl VersionsClient {
    pub fn do_stuff(&self) -> Result<()> {
        let generic_local_pwsg =
            "https://artifactory.it.keysight.com/artifactory/api/storage/generic-local-pwsg/siggen";
        let response = self
            .client
            .get(format!("{}/packages-linux/develop", generic_local_pwsg))
            .send()?;
        let temp: ArtifactoryDirectory = serde_json::from_str(&response.text()?)?;
        dbg!(&temp);

        for child in temp.children {
            println!("{}", child.uri);
        }

        Ok(())
    }

    pub fn download(
        &self,
        url: String,
        file_name: String,
        status: Arc<Mutex<DownloadStatus>>,
        repaint: Arc<dyn RepaintSignal>,
    ) -> Result<()> {
        let destination_dir = dirs::download_dir().unwrap_or(dirs::home_dir().ok_or(
            anyhow::Error::msg("Could not find Downloads or Home directories"),
        )?);

        let client = self.client.clone();
        std::thread::spawn(move || {
            {
                let mut locked = status.lock().unwrap();
                *locked = DownloadStatus::Downloading;
            }
            match download_internal(&client, &url, &destination_dir, &file_name) {
                Ok(_) => {
                    let mut locked = status.lock().unwrap();
                    *locked = DownloadStatus::Idle;
                }
                Err(_) => {
                    let mut locked = status.lock().unwrap();
                    *locked = DownloadStatus::Error;
                }
            }
            repaint.request_repaint();
        });

        Ok(())
    }

    pub fn get_packages_info(&self, branch: &String) -> ArtifactoryDirectory {
        let request = self.client.get(format!(
            "{}/{}/{}",
            BASE_API_URL,
            package_segments(),
            branch
        ));

        match request.send() {
            Ok(response) => {
                serde_json::from_str(&response.text().unwrap_or_default()).unwrap_or_default()
            }
            Err(_) => ArtifactoryDirectory::default(),
        }
    }

    pub fn get_branch_names(&self) -> BTreeSet<String> {
        let request = self
            .client
            .get(format!("{}/{}", BASE_API_URL, package_segments()));
        let mut result = BTreeSet::new();
        match request.send() {
            Ok(response) => {
                let response: ArtifactoryDirectory =
                    serde_json::from_str(&response.text().unwrap_or_default()).unwrap_or_default();
                for child in response.children {
                    result.insert(child.uri.trim_start_matches("/").to_string());
                }
            }
            Err(_) => {}
        };
        result
    }
}

fn download_internal(
    client: &Arc<reqwest::blocking::Client>,
    url: &String,
    destination_dir: &PathBuf,
    file_name: &String,
) -> Result<()> {
    let mut out = File::create(format!("{}/{}", destination_dir.display(), file_name))?;
    client.get(url).send()?.copy_to(&mut out)?;
    Ok(())
}

pub fn package_segments() -> String {
    format!(
        "{}/{}",
        "generic-local-pwsg/siggen",
        if cfg!(windows) {
            "packages"
        } else {
            "packages-linux"
        }
    )
}

pub fn develop_branch() -> String {
    DEVELOP_BRANCH.to_string()
}

pub const DEVELOP_BRANCH: &str = "develop";
pub const BASE_DOWNLOAD_URL: &'static str = "https://artifactory.it.keysight.com/artifactory";
pub const BASE_API_URL: &'static str =
    "https://artifactory.it.keysight.com/artifactory/api/storage";
