use crate::AGRM_NAME;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
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

impl Clone for GlobalSettings {
    fn clone(&self) -> Self {
        Self {
            structure: self.structure.clone(),
            root: self.root.clone(),
        }
    }
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

impl GlobalSettings {
    pub fn merge(mut self, target: Option<GlobalSettings>) -> GlobalSettings {
        if let Some(target) = target {
            self.structure = self.structure.or(target.structure);
            self.root = self.root.or(target.root);
        }
        self
    }
}
