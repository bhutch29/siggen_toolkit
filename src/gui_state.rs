use crate::cli::SimulatedChannel;
use crate::logging::LoggingConfiguration;
use crate::versions::{
    develop_branch, parse_semver, DownloadStatus, FileInfo, SemVer, VersionsClient,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct HwconfigState {
    pub channel_count: u8,
    pub platform: SimulatedChannel,
    pub write_error: bool,
    pub remove_error: bool,
}

impl Default for HwconfigState {
    fn default() -> Self {
        Self {
            channel_count: 1,
            platform: SimulatedChannel::MCS31 { signal_count: 1 },
            write_error: false,
            remove_error: false,
        }
    }
}

#[derive(Default)]
pub struct LoggingState {
    pub config: LoggingConfiguration,
    pub custom_path: String,
    pub loaded_from: Option<PathBuf>,
    pub write_error: bool,
    pub remove_error: bool,
}

/// Recursive data structure. Intended to hold Major, Minor, and Patch versions as keys in nested maps
#[derive(Debug, Default, Clone)]
pub struct FilterOptions {
    pub next: BTreeMap<u16, FilterOptions>,
}

impl FilterOptions {
    pub fn new(next: BTreeMap<u16, FilterOptions>) -> Self {
        Self { next }
    }
}

#[derive(Default)]
pub struct ReportsState {
    pub name: String,
    pub previous_name: String,
    pub log_file_path: Option<PathBuf>,
    pub log_cfg_path: Option<PathBuf>,
    pub hwconfig_path: Option<PathBuf>,
    pub installed_version: Option<String>,
    pub generate_status: Option<bool>,
    pub file_exists: bool,
    pub upload_status: Arc<Mutex<DownloadStatus>>
}

impl ReportsState {
    pub fn name_changed(&mut self) -> bool {
        if self.previous_name != self.name {
            self.previous_name = self.name.clone();
            return true;
        }
        false
    }
}

#[derive(Default, Clone)]
pub struct VersionsFilter {
    pub options: FilterOptions,
    pub major_filter: Option<u16>,
    pub minor_filter: Option<u16>,
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
        self.sort_cache_for(branch);
        self.populate_filter_options_for(branch);
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

    pub fn sort_cache_for(&mut self, branch: &String) {
        if let Some(files) = self.cache.get_mut(branch) {
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

    pub fn populate_filter_options_for(&mut self, branch: &String) {
        if let Some(files) = self.cache.get_mut(branch) {
            let mut filter = VersionsFilter::default();
            files
                .iter()
                .filter_map(|file| parse_semver(&file.version))
                .for_each(|semver| {
                    VersionsState::populate_filter_with_one_version(&semver, &mut filter.options)
                });
            self.filters.insert(branch.clone(), filter);
        }
    }

    fn populate_filter_with_one_version(version: &SemVer, options: &mut FilterOptions) {
        let mut patch = BTreeMap::new();
        patch.insert(version.patch, FilterOptions::default());
        let mut minor = BTreeMap::new();
        minor.insert(version.minor, FilterOptions::new(patch.clone()));

        match options.next.get_mut(&version.major) {
            None => {
                options
                    .next
                    .insert(version.major, FilterOptions::new(minor));
            }
            Some(ref mut major) => match major.next.get_mut(&version.minor) {
                None => {
                    major.next.insert(version.minor, FilterOptions::new(patch));
                }
                Some(ref mut minor) => {
                    minor
                        .next
                        .entry(version.patch)
                        .or_insert(FilterOptions::default());
                }
            },
        }
    }

    pub fn filter_match(&self, version: &String) -> bool {
        let mut matched = true;
        if let (Some(version), Some(filter)) = (parse_semver(version), self.get_current_filter()) {
            if let Some(major) = filter.major_filter {
                matched &= version.major == major;
            }
            if let Some(minor) = filter.minor_filter {
                matched &= version.minor == minor;
            }
            if let Some(patch) = filter.patch_filter {
                matched &= version.patch == patch;
            }
        }
        matched
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
