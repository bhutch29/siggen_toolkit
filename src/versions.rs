use eframe::epi::RepaintSignal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct FileInfo {
    pub full_name: String,
    pub version: String,
    pub date: String,
}

#[derive(PartialEq, Clone)]
pub enum RequestStatus {
    Idle,
    InProgress,
    Success,
    Error,
}

impl Default for RequestStatus {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Default, PartialEq)]
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
    let parts: Vec<Option<u16>> = semver.split('-').map(|x| x.parse::<u16>().ok()).take(4).collect();

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

    fn make_semver(major: u16, minor: u16, patch: u16, prerelease: Option<u16>) -> versions::SemVer {
        versions::SemVer {
            major,
            minor,
            patch,
            prerelease,
        }
    }

    #[test]
    fn parse() {
        assert_eq!(parse_semver(&"1-2-3-4".to_string()).unwrap().prerelease, Some(4));
        assert_eq!(parse_semver(&"1-2-3".to_string()).unwrap().prerelease, None);
        assert_eq!(parse_semver(&"1-2-3-4-5".to_string()).unwrap().prerelease, Some(4));
        assert_eq!(parse_semver(&"1-2".to_string()).is_none(), true);
        assert_eq!(parse_semver(&"1-2-l".to_string()).is_none(), true);
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
    // TODO: download to directories by branch
    pub fn download_package(
        &self,
        branch: &str,
        file_name: &str,
        status: Arc<Mutex<RequestStatus>>,
        repaint: Arc<dyn RepaintSignal>,
    ) -> anyhow::Result<()> {
        let url = format!("{}/{}/{}/{}", BASE_FILE_URL, package_segments(), branch, file_name);

        let destination_dir = dirs::download_dir()
            .unwrap_or(dirs::home_dir().ok_or(anyhow::anyhow!("Could not find Downloads or Home directories"))?);

        let file_name = file_name.to_string();
        let client = self.client.clone();
        std::thread::spawn(move || {
            {
                *status.lock().unwrap() = RequestStatus::InProgress;
            }
            match download_internal(&client, &url, &destination_dir, &file_name) {
                Ok(_) => {
                    *status.lock().unwrap() = RequestStatus::Success;
                }
                Err(_) => {
                    *status.lock().unwrap() = RequestStatus::Error;
                }
            }
            repaint.request_repaint();
        });

        Ok(())
    }

    pub fn upload_report(
        &self,
        path: &Path,
        status: Option<Arc<Mutex<RequestStatus>>>,
        repaint: Option<Arc<dyn RepaintSignal>>,
    ) -> anyhow::Result<()> {
        let url = format!(
            "{}/{}/{}",
            BASE_FILE_URL,
            report_segments(),
            path.file_name().unwrap().to_string_lossy()
        );

        let client = self.client.clone();
        let path = PathBuf::from(path);
        std::thread::spawn(move || {
            if let Some(ref status) = status {
                *status.lock().unwrap() = RequestStatus::InProgress;
            }
            match upload_report_internal(&client, &url, &path) {
                Ok(_) if status.is_some() => {
                    *status.unwrap().lock().unwrap() = RequestStatus::Success;
                }
                Err(_) if status.is_some() => {
                    *status.unwrap().lock().unwrap() = RequestStatus::Error;
                }
                _ => {}
            }
            if let Some(repaint) = repaint {
                repaint.request_repaint();
            }
        });

        Ok(())
    }

    pub fn get_packages_info(&self, branch: &str) -> Vec<FileInfo> {
        self.get_info(branch, &package_segments())
            .into_iter()
            .filter_map(|full_name| {
                let mut split: VecDeque<String> = full_name
                    .trim_start_matches("siggen_")
                    .trim_end_matches(".zip")
                    .trim_end_matches("_linux")
                    .split('_')
                    .take(2)
                    .map(|s| s.to_string())
                    .collect();

                if let (Some(version), Some(date)) = (split.pop_front(), split.pop_front()) {
                    Some(FileInfo {
                        full_name,
                        version,
                        date,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_installers_info(&self, _branch: &str) -> Vec<FileInfo> {
        // let info = self.get_info(branch, &installer_segments());
        Vec::new()
        // TODO:
        // let mut result = Vec::new();
        // for full_name in info {
        //     let formatted = full_name
        //         .trim_start_matches("siggen_")
        //         .trim_end_matches(".zip")
        //         .trim_end_matches("_linux");
        //     let split: [String; 2] = formatted.split("_").take(2).collect();
        //
        //     result.push(FileInfo {
        //         full_name,
        //         version: split[0].clone(),
        //         date: split[1].clone(),
        //     });
        // }
        // result
    }

    fn get_info(&self, branch: &str, segments: &str) -> Vec<String> {
        parse_children(
            self.api_request(&format!("{}/{}", segments, branch))
                .unwrap_or_default(),
        )
    }

    pub fn get_packages_branch_names(&self) -> Vec<String> {
        self.get_branch_names(&package_segments())
    }

    pub fn get_installers_branch_names(&self) -> Vec<String> {
        self.get_branch_names(&installer_segments())
    }

    fn get_branch_names(&self, segments: &str) -> Vec<String> {
        parse_children(self.api_request(segments).unwrap_or_default())
    }

    fn api_request(&self, segments: &str) -> Option<ArtifactoryDirectory> {
        let request = self.client.get(format!("{}/{}", BASE_API_URL, segments));
        serde_json::from_str(&request.send().ok()?.text().unwrap_or_default()).ok()?
    }
}

fn parse_children(response: ArtifactoryDirectory) -> Vec<String> {
    response
        .children
        .iter()
        .map(|child| child.uri.trim_start_matches('/').to_string())
        .collect()
}

fn download_internal(
    client: &Arc<reqwest::blocking::Client>,
    url: &str,
    destination_dir: &Path,
    file_name: &str,
) -> anyhow::Result<()> {
    let mut out = std::fs::File::create(format!("{}/{}", destination_dir.display(), file_name))?;
    client.get(url).send()?.error_for_status()?.copy_to(&mut out)?;
    Ok(())
}

fn upload_report_internal(client: &Arc<reqwest::blocking::Client>, url: &str, path: &Path) -> anyhow::Result<()> {
    let file = std::fs::File::open(path)?;
    client.put(url).body(file).send()?.error_for_status()?;
    Ok(())
}

pub fn installed_version() -> Option<String> {
    //TODO:
    None
}

pub fn package_segments() -> String {
    format!(
        "{}/{}",
        "generic-local-pwsg/siggen",
        if cfg!(windows) { "packages" } else { "packages-linux" }
    )
}

pub fn installer_segments() -> String {
    "generic-local-boxer-releases/siggen".to_string()
}

pub fn report_segments() -> String {
    "generic-local-pwsg/siggen/reports".to_string()
}

pub fn develop_branch() -> String {
    DEVELOP_BRANCH.to_string()
}

pub const DEVELOP_BRANCH: &str = "develop";
pub const BASE_FILE_URL: &str = "https://artifactory.it.keysight.com/artifactory";
pub const BASE_API_URL: &str = "https://artifactory.it.keysight.com/artifactory/api/storage";
