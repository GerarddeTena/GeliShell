pub mod registry;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EcosystemCatalog {
    pub meta: EcosystemMeta,
    #[serde(default)]
    pub ops: Vec<EcosystemOperation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EcosystemMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub levels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EcosystemOperation {
    pub operation: String,
    pub level: String,
    #[serde(default)]
    pub commands: Vec<EcosystemCommand>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EcosystemCommand {
    pub subsystem: String,
    pub command: String,
}

