use crate::cli::SimulatedChannel;
use crate::logging::LoggingConfiguration;
use crate::versions::{develop_branch, parse_semver, DownloadStatus, FileInfo, VersionsClient};
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct HwconfigState {
    pub channel_count: u8,
    pub platform: SimulatedChannel,
    pub write_error: bool,
    pub remove_error: bool,
}

#[derive(Default)]
pub struct LoggingState {
    pub config: LoggingConfiguration,
    pub custom_path: String,
    pub loaded_from: Option<PathBuf>,
    pub write_error: bool,
    pub remove_error: bool,
}

#[derive(Default, Clone)]
pub struct VersionsFilter {
    pub major_filter_options: BTreeSet<u16>,
    pub major_filter: Option<u16>,
    pub minor_filter_options: BTreeSet<u16>,
    pub minor_filter: Option<u16>,
    pub patch_filter_options: BTreeSet<u16>,
    pub patch_filter: Option<u16>,
}

pub enum VersionsTypes {
    Packages,
    Installers,
}

impl Default for VersionsTypes {
    fn default() -> Self {
        Self::Packages
    }
}

#[derive(Default)]
pub struct VersionsState {
    pub client: VersionsClient,
    pub branch_names: Vec<String>,
    pub selected_branch: String,
    pub status: HashMap<(String, FileInfo), Arc<Mutex<DownloadStatus>>>,
    pub which: VersionsTypes,

    filters: HashMap<String, VersionsFilter>,
    cache: HashMap<String, Vec<FileInfo>>,
    already_setup: bool,
}

impl VersionsState {
    pub fn new(which: VersionsTypes) -> Self {
        Self {
            which,
            selected_branch: develop_branch(),
            ..Self::default()
        }
    }

    pub fn setup_if_needed(&mut self) {
        if !self.already_setup {
            self.update_branch_names();
            self.update_cache(&develop_branch());
            self.already_setup = true;
        }
    }

    fn update_branch_names(&mut self) {
        self.branch_names = match &self.which {
            VersionsTypes::Packages => self.client.get_packages_branch_names(),
            VersionsTypes::Installers => self.client.get_installers_branch_names(),
        };
    }

    pub fn refresh(&mut self) {
        self.update_branch_names();
        self.cache.clear();
        self.update_current_cache_if_needed();
    }

    pub fn update_current_cache_if_needed(&mut self) {
        if self.get_current_cache().is_none() {
            self.update_cache(&self.selected_branch.clone());
        }
    }

    fn update_cache(&mut self, branch: &String) {
        let info = match &self.which {
            VersionsTypes::Packages => self.client.get_packages_info(branch),
            VersionsTypes::Installers => self.client.get_installers_info(branch),
        };
        self.cache.insert(branch.clone(), info);
        // TODO: these do a lot, we could do less?
        self.sort_cache();
        self.populate_filter_options();
    }

    pub fn get_current_filter(&self) -> Option<&VersionsFilter> {
        self.filters.get(&self.selected_branch)
    }

    pub fn get_current_filter_mut(&mut self) -> Option<&mut VersionsFilter> {
        self.filters.get_mut(&self.selected_branch)
    }

    pub fn get_current_cache(&self) -> Option<&Vec<FileInfo>> {
        self.cache.get(&self.selected_branch)
    }

    pub fn latest(&self) -> Option<FileInfo> {
        self.cache
            .get(&self.selected_branch)
            .and_then(|files| files.last().cloned())
    }

    pub fn sort_cache(&mut self) {
        for (_, files) in &mut self.cache {
            files.sort_by(|a, b| {
                let parsed_a = parse_semver(&a.version);
                let parsed_b = parse_semver(&b.version);

                if parsed_a.is_some() && parsed_b.is_some() {
                    return parsed_a.unwrap().partial_cmp(&parsed_b.unwrap()).unwrap();
                }

                if parsed_a.is_none() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            });
        }
    }

    pub fn populate_filter_options(&mut self) {
        self.filters.clear();

        for (branch, files) in &mut self.cache {
            let mut filter = VersionsFilter::default();
            for file in files {
                let semver = parse_semver(&file.version);
                if let Some(v) = semver {
                    filter.major_filter_options.insert(v.major);
                    filter.minor_filter_options.insert(v.minor);
                    filter.patch_filter_options.insert(v.patch);
                }
            }
            self.filters.insert(branch.clone(), filter);
        }
    }

    pub fn filter_match(&self, version: &String) -> bool {
        if let (Some(version), Some(filter)) = (parse_semver(version), self.get_current_filter()) {
            if let Some(major) = filter.major_filter {
                if version.major != major {
                    return false;
                }
            }
            if let Some(minor) = filter.minor_filter {
                if version.minor != minor {
                    return false;
                }
            }
            if let Some(patch) = filter.patch_filter {
                if version.patch != patch {
                    return false;
                }
            }
        }
        true
    }

    pub fn get_package_download_status(&self, file_info: &FileInfo) -> DownloadStatus {
        let status = self
            .status
            .get(&(self.selected_branch.clone(), file_info.clone()));

        match status {
            None => DownloadStatus::Idle,
            Some(status) => status.lock().unwrap().clone(),
        }
    }
}
