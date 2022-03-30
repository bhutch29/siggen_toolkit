use serde::{Deserialize, Serialize};
use std::path::Path;

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

#[derive(Serialize, Deserialize, Clone, Debug)]
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

    #[serde(default = "as_true")]
    pub this_setting_resolve_enabled: bool,
    #[serde(default = "as_true")]
    pub this_setting_set_enabled: bool,
    #[serde(default = "as_true")]
    pub this_setting_set_by_user_enabled: bool,
    #[serde(default = "as_true")]
    pub this_setting_marked_enabled: bool,
    #[serde(default = "as_true")]
    pub this_setting_op_value_updated: bool,

    #[serde(default)]
    pub setting_registered_with_gui: bool,
    #[serde(default)]
    pub setting_log_control_seos: bool,
}

impl Default for SettingsGlobal {
    fn default() -> Self {
        Self {
            all_enabled: false,
            setting_resolve_enabled: false,
            setting_set_enabled: false,
            setting_set_by_user_enabled: false,
            setting_marked_enabled: false,
            setting_op_value_updated: false,
            this_setting_resolve_enabled: true,
            this_setting_set_enabled: true,
            this_setting_set_by_user_enabled: true,
            this_setting_marked_enabled: true,
            this_setting_op_value_updated: true,
            setting_registered_with_gui: false,
            setting_log_control_seos: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OperationsInstance {
    pub names: Vec<String>,
    pub flags: OperationsInstanceFlags,
}

impl Default for OperationsInstance {
    fn default() -> Self {
        Self {
            names: vec![String::default()],
            flags: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    #[serde(default = "as_negative_one")]
    pub break_on_mark_after_n: i32,
    #[serde(default)]
    pub break_on_abort: bool,
}

impl Default for OperationsInstanceFlags {
    fn default() -> Self {
        Self {
            trace_all: false,
            trace_on_mark: false,
            trace_on_resolve: false,
            trace_on_abort: false,
            trace_on_remove: false,
            trace_on_add: false,
            trace_on_bind: false,
            break_on_mark: false,
            break_on_mark_after_n: -1,
            break_on_abort: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SettingsInstance {
    pub setting_paths: Vec<String>,
    pub flags: SettingsInstanceFlags,
}

impl Default for SettingsInstance {
    fn default() -> Self {
        Self {
            setting_paths: vec![String::default()],
            flags: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInstanceFlags {
    #[serde(default)]
    pub trace_enabled: bool,
    #[serde(default)]
    pub break_on_set: bool,
    #[serde(default = "as_negative_one")]
    pub break_on_set_after_n: i32,
    #[serde(default)]
    pub break_on_set_by_user: bool,
    #[serde(default = "as_negative_one")]
    pub break_on_set_by_user_after_n: i32,
    #[serde(default)]
    pub break_on_marked: bool,
    #[serde(default = "as_negative_one")]
    pub break_on_marked_after_n: i32,
    #[serde(default)]
    pub break_on_resolve: bool,
    #[serde(default = "as_negative_one")]
    pub break_on_resolve_after_n: i32,
}

impl Default for SettingsInstanceFlags {
    fn default() -> Self {
        Self {
            trace_enabled: false,
            break_on_set: false,
            break_on_set_after_n: -1,
            break_on_set_by_user: false,
            break_on_set_by_user_after_n: -1,
            break_on_marked: false,
            break_on_marked_after_n: -1,
            break_on_resolve: false,
            break_on_resolve_after_n: -1,
        }
    }
}

pub fn as_true() -> bool {
    true
}
pub fn as_negative_one() -> i32 {
    -1
}

pub const FILE_NAME: &str = "ionDebug.json";
pub const ENV_VAR: &str = "ION_DEBUG_DIR";
pub const CONFLUENCE_URL: &str = "https://confluence.it.keysight.com/display/PWL/Ion+Diagnostics";

pub fn get_config_from(path: &Path) -> Option<DiagnosticsConfiguration> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
}

pub fn set_config(path: &Path, config: DiagnosticsConfiguration) -> anyhow::Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}
