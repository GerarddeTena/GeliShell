use super::EcosystemCatalog;
use std::collections::HashMap;

static CARGO: &str = include_str!("../../../../commands/ecosystems/cargo-lang.toml");
static DOCKER: &str = include_str!("../../../../commands/ecosystems/docker.toml");
static DOTNET: &str = include_str!("../../../../commands/ecosystems/dotnet.toml");
static GIT: &str = include_str!("../../../../commands/ecosystems/git.toml");
static NODE: &str = include_str!("../../../../commands/ecosystems/node.toml");
static NPM: &str = include_str!("../../../../commands/ecosystems/npm.toml");
static PNPM: &str = include_str!("../../../../commands/ecosystems/pnpm.toml");
static PYTHON: &str = include_str!("../../../../commands/ecosystems/python.toml");
static TYPESCRIPT: &str = include_str!("../../../../commands/ecosystems/typescript.toml");

const AVAILABLE: &[&str] = &[
    "cargo",
    "docker",
    "dotnet",
    "git",
    "npm",
    "python",
    "typescript",
];

#[derive(Debug, thiserror::Error)]
pub enum EcosystemError {
    #[error("failed to deserialize ecosystem '{name}': {source}")]
    Deserialize {
        name: &'static str,
        #[source]
        source: toml::de::Error,
    },

    #[error("ecosystem '{name}' has no operations")]
    EmptyOps { name: String },
}

pub struct EcosystemRegistry {
    ecosystems: HashMap<&'static str, EcosystemCatalog>,
}

impl EcosystemRegistry {
    pub fn load() -> Result<Self, EcosystemError> {
        let mut ecosystems = HashMap::with_capacity(AVAILABLE.len());
        ecosystems.insert("cargo", parse_catalog("cargo", CARGO)?);
        ecosystems.insert("docker", parse_catalog("docker", DOCKER)?);
        ecosystems.insert("dotnet", parse_catalog("dotnet", DOTNET)?);
        ecosystems.insert("git", parse_catalog("git", GIT)?);
        ecosystems.insert("node", parse_catalog("node", NODE)?);
        ecosystems.insert("npm", parse_catalog("npm", NPM)?);
        ecosystems.insert("pnpm", parse_catalog("pnpm", PNPM)?);
        ecosystems.insert("python", parse_catalog("python", PYTHON)?);
        ecosystems.insert("typescript", parse_catalog("typescript", TYPESCRIPT)?);

        Ok(Self { ecosystems })
    }

    pub fn get(&self, name: &str) -> Option<&EcosystemCatalog> {
        self.ecosystems.get(name)
    }

    pub fn available() -> &'static [&'static str] {
        AVAILABLE
    }
}

fn parse_catalog(name: &'static str, raw: &str) -> Result<EcosystemCatalog, EcosystemError> {
    let catalog = toml::from_str::<EcosystemCatalog>(raw)
        .map_err(|source| EcosystemError::Deserialize { name, source })?;

    if catalog.ops.is_empty() {
        return Err(EcosystemError::EmptyOps {
            name: catalog.meta.name,
        });
    }

    Ok(catalog)
}
