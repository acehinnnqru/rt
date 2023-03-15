use serde::Deserialize;

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
    dev: bool,
    structure: Vec<StructureNode>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        use StructureNode::*;
        Self {
            dev: false,
            structure: vec![Platform, NameSpace, Name],
        }
    }
}
