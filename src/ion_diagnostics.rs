use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::common::in_cwd;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct DiagnosticsConfiguration {
    pub operations: Operations,
    pub settings: Settings,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Operations {
    pub global: OperationsGlobal,
    pub instance: Vec<OperationsInstance>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Settings {
    pub global: SettingsGlobal,
    pub instance: Vec<SettingsInstance>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OperationsGlobal {
    #[serde(default)]
    pub trace_all: bool,
    #[serde(default)]
    pub trace_on_mark: bool,
    #[serde(default)]
    pub trace_on_resolve: bool,
    #[serde(default)]
    pub trace_on_abort: bool,
    #[serde(default)]
    pub trace_on_remove: bool,
    #[serde(default)]
    pub trace_on_add: bool,
    #[serde(default)]
    pub trace_on_bind: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettingsGlobal {
    #[serde(default)]
    pub all_enabled: bool,

    #[serde(default)]
    pub setting_resolve_enabled: bool,
    #[serde(default)]
    pub setting_set_enabled: bool,
    #[serde(default)]
    pub setting_set_by_user_enabled: bool,
    #[serde(default)]
    pub setting_marked_enabled: bool,
    #[serde(default)]
    pub setting_op_value_updated: bool,

    #[serde(default="as_true")]
    pub this_setting_resolve_enabled: bool,
    #[serde(default="as_true")]
    pub this_setting_set_enabled: bool,
    #[serde(default="as_true")]
    pub this_setting_set_by_user_enabled: bool,
    #[serde(default="as_true")]
    pub this_setting_marked_enabled: bool,
    #[serde(default="as_true")]
    pub this_setting_op_value_updated: bool,

    #[serde(default)]
    pub setting_registered_with_gui: bool,
    #[serde(default)]
    pub setting_log_control_seos: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OperationsInstance {
    pub names: Vec<String>,
    pub flags: OperationsInstanceFlags,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct OperationsInstanceFlags {
    #[serde(default)]
    pub trace_all: bool,
    #[serde(default)]
    pub trace_on_mark: bool,
    #[serde(default)]
    pub trace_on_resolve: bool,
    #[serde(default)]
    pub trace_on_abort: bool,
    #[serde(default)]
    pub trace_on_remove: bool,
    #[serde(default)]
    pub trace_on_add: bool,
    #[serde(default)]
    pub trace_on_bind: bool,

    #[serde(default)]
    pub break_on_mark: bool,
    pub break_on_mark_after_n: Option<u16>,
    #[serde(default)]
    pub break_on_abort: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SettingsInstance {
    pub setting_paths: Vec<String>,
    pub flags: SettingsInstanceFlags,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInstanceFlags {
    #[serde(default)]
    pub trace_enabled: bool,
    #[serde(default)]
    pub break_on_set: bool,
    #[serde(default="as_negative_one")]
    pub break_on_set_after_n: i32,
    #[serde(default)]
    pub break_on_set_by_user: bool,
    pub break_on_set_by_user_after_n: Option<i32>,
    #[serde(default)]
    pub break_on_marked: bool,
    pub break_on_marked_after_n: Option<i32>,
    #[serde(default)]
    pub break_on_resolve: bool,
    pub break_on_resolve_after_n: Option<i32>,
}

impl Default for SettingsInstanceFlags {
    fn default() -> Self {
        Self {
            trace_enabled: false,
            break_on_set: false,
            break_on_set_after_n: -1,
            break_on_set_by_user: false,
            break_on_set_by_user_after_n: None,
            break_on_marked: false,
            break_on_marked_after_n: None,
            break_on_resolve: false,
            break_on_resolve_after_n: None,
        }
    }
}

pub fn as_true() -> bool {true}
pub fn as_negative_one() -> i32 {-1}

pub const FILE_NAME: &str = "ionDebug.json";

pub fn valid_paths() -> Vec<PathBuf> {
    Vec::from([PathBuf::from("test")])
}

pub fn get_path() -> Option<PathBuf> {
    for path in valid_paths() {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn get_path_or_cwd() -> PathBuf {
    get_path().unwrap_or_else(|| in_cwd(FILE_NAME))
}

pub fn get_config_from(path: &Path) -> Option<DiagnosticsConfiguration> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

pub fn set_config(path: &Path, config: DiagnosticsConfiguration) -> anyhow::Result<()> {
    println!("{:?}", config);
    std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}
