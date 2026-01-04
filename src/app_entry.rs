use crate::app_id::AppId;
use crate::app_info::AppInfo;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct AppEntry {
    pub backend_name: &'static str,
    pub info: Arc<AppInfo>,
    pub installed: bool,
}

pub type Apps = HashMap<AppId, Vec<AppEntry>>;
