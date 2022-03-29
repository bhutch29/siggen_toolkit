use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DiagnosticsConfiguration {
    pub operations: Operations,
    pub settings: Settings,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Operations {
    pub global: OperationsGlobal,
    pub instance: Vec<OperationsInstance>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Settings {
    pub global: SettingsGlobal,
    pub instance: Vec<SettingsInstance>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OperationsGlobal {
    pub trace_all: Option<bool>,
    pub trace_on_mark: Option<bool>,
    pub trace_on_resolve: Option<bool>,
    pub trace_on_abort: Option<bool>,
    pub trace_on_remove: Option<bool>,
    pub trace_on_add: Option<bool>,
    pub trace_on_bind: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SettingsGlobal {
    pub all_enabled: Option<bool>,

    pub setting_resolve_enabled: Option<bool>,
    pub setting_set_enabled: Option<bool>,
    pub setting_set_by_user_enabled: Option<bool>,
    pub setting_marked_enabled: Option<bool>,
    pub setting_op_value_updated: Option<bool>,

    pub this_setting_resolve_enabled: Option<bool>,
    pub this_setting_set_enabled: Option<bool>,
    pub this_setting_set_by_user_enabled: Option<bool>,
    pub this_setting_marked_enabled: Option<bool>,
    pub this_setting_op_value_updated: Option<bool>,

    pub setting_registered_with_gui: Option<bool>,
    pub setting_log_control_seos: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OperationsInstance {
    pub names: Vec<String>,
    pub flags: OperationsInstanceFlags,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct OperationsInstanceFlags {
    pub trace_all: Option<bool>,
    pub trace_on_mark: Option<bool>,
    pub trace_on_resolve: Option<bool>,
    pub trace_on_abort: Option<bool>,
    pub trace_on_remove: Option<bool>,
    pub trace_on_add: Option<bool>,
    pub trace_on_bind: Option<bool>,

    pub break_on_mark: Option<bool>,
    pub break_on_mark_after_n: Option<u16>,
    pub break_on_abort: Option<bool>
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SettingsInstance {
    pub setting_paths: Vec<String>,
    pub flags: SettingsInstanceFlags,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SettingsInstanceFlags {
    pub trace_enabled: Option<bool>,
    pub break_on_set: Option<bool>,
    pub break_on_set_after_n: Option<u16>,
    pub break_on_set_by_user: Option<bool>,
    pub break_on_set_by_user_after_n: Option<u16>,
    pub break_on_marked: Option<bool>,
    pub break_on_marked_after_n: Option<u16>,
    pub break_on_resolve: Option<bool>,
    pub break_on_resolve_after_n: Option<u16>,
}
