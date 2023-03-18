use serde::Deserialize;
use crate::AGRM_NAME;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StructureNode {
    OSUser,
    Platform,
    NameSpace,
    Name,
    Branch,
    Tag,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct GlobalSettings {
    pub structure: Option<Vec<StructureNode>>,
    pub root: Option<String>,
}

#[cfg(target_family = "windows")]
fn os_default_root() -> String {
    std::path::Path::new(std::env::var("USERPROFILE"))
        .join(format!(".{}", AGRM_NAME))
        .join("root")
        .to_string_lossy()
        .to_string()
}

#[cfg(target_family = "unix")]
fn os_default_root() -> String {
    std::path::Path::new(&std::env::var("HOME").unwrap())
        .join(format!(".{}", AGRM_NAME))
        .join("root")
        .to_string_lossy()
        .to_string()
}

impl Default for GlobalSettings {
    fn default() -> Self {
        use StructureNode::*;
        Self {
            root: Some(os_default_root()),
            structure: Some(vec![Platform, NameSpace, Name]),
        }
    }
}
