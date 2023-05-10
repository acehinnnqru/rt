use crate::settings::global::StructureNode;
use crate::settings::Settings;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RepRoot {
    root: String,
    structure: Vec<StructureNode>,
}

impl From<&Settings> for RepRoot {
    fn from(value: &Settings) -> Self {
        let global = value.global.clone().unwrap();
        Self {
            root: global.root.unwrap(),
            structure: global.structure.unwrap(),
        }
    }
}

impl RepRoot {
    pub fn get_root(&self) -> &str {
        &self.root
    }

    pub fn get_structure(&self) -> &Vec<StructureNode> {
        &self.structure
    }

    pub fn gen_target_path(&self, platform: &str, namespace: &str, name: &str) -> PathBuf {
        let mut path = PathBuf::from(self.get_root());
        path.push(platform);
        path.push(namespace);
        path.push(name);
        path
    }
}
