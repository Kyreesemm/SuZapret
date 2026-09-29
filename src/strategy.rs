use std::{fs, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::{Result, SuZapretError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub metadata: Metadata,
    pub intercept: Intercept,
    #[serde(rename = "profile", default)]
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    pub name: String,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intercept {
    #[serde(default)]
    pub tcp_ports: Vec<String>,
    #[serde(default)]
    pub udp_ports: Vec<String>,
    #[serde(default)]
    pub game_filter_tcp: bool,
    #[serde(default)]
    pub game_filter_udp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub protocol: Protocol,
    pub ports: Vec<String>,
    #[serde(default)]
    pub game_filter: bool,
    #[serde(default)]
    pub hostlists: Vec<PathBuf>,
    #[serde(default)]
    pub hostlist_domains: Vec<String>,
    #[serde(default)]
    pub ipsets: Vec<PathBuf>,
    #[serde(default)]
    pub ipset_excludes: Vec<PathBuf>,
    #[serde(default)]
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Strategy {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let source = fs::read_to_string(path)?;
        let strategy: Self = toml::from_str(&source)?;
        strategy.validate()?;
        Ok(strategy)
    }

    pub fn validate(&self) -> Result<()> {
        if self.metadata.version != 1 {
            return Err(SuZapretError::InvalidStrategy(format!(
                "unsupported format version {}",
                self.metadata.version
            )));
        }
        if self.metadata.id.is_empty() || self.metadata.name.is_empty() {
            return Err(SuZapretError::InvalidStrategy(
                "metadata.id and metadata.name are required".into(),
            ));
        }
        if self.intercept.tcp_ports.is_empty() && self.intercept.udp_ports.is_empty() {
            return Err(SuZapretError::InvalidStrategy(
                "at least one intercepted port is required".into(),
            ));
        }
        if self.profiles.is_empty() {
            return Err(SuZapretError::InvalidStrategy(
                "at least one profile is required".into(),
            ));
        }
        for profile in &self.profiles {
            if profile.id.is_empty() || profile.ports.is_empty() {
                return Err(SuZapretError::InvalidStrategy(
                    "each profile needs an id and at least one port".into(),
                ));
            }
            if profile.arguments.iter().any(|arg| arg.contains('\n') || arg.contains('\r')) {
                return Err(SuZapretError::InvalidStrategy(
                    "profile arguments must be single-line values".into(),
                ));
            }
        }
        Ok(())
    }
}

fn default_version() -> u32 { 1 }

#[derive(Debug, Default)]
pub struct StrategyStore {
    root: PathBuf,
}

impl StrategyStore {
    pub fn new(root: impl Into<PathBuf>) -> Self { Self { root: root.into() } }

    pub fn load(&self, id: &str) -> Result<Strategy> {
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return Err(SuZapretError::InvalidStrategy("invalid strategy id".into()));
        }
        Strategy::from_file(self.root.join(format!("{id}.toml")))
    }

    pub fn list(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
                if let Some(stem) = path.file_stem().and_then(|name| name.to_str()) {
                    ids.push(stem.to_owned());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::{SystemTime, UNIX_EPOCH}};

    use super::*;

    fn temporary_directory() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("suzapret-strategy-test-{suffix}"))
    }

    fn valid_strategy() -> Strategy {
        Strategy {
            metadata: Metadata { id: "test".into(), name: "Test".into(), version: 1, source: String::new() },
            intercept: Intercept { tcp_ports: vec!["443".into()], udp_ports: Vec::new(), game_filter_tcp: false, game_filter_udp: false },
            profiles: vec![Profile {
                id: "tls".into(), protocol: Protocol::Tcp, ports: vec!["443".into()], game_filter: false,
                hostlists: Vec::new(), hostlist_domains: Vec::new(), ipsets: Vec::new(), ipset_excludes: Vec::new(),
                arguments: vec!["--dpi-desync=fake".into()],
            }],
        }
    }

    #[test]
    fn validates_a_complete_strategy() {
        assert!(valid_strategy().validate().is_ok());
    }

    #[test]
    fn rejects_unsupported_format_versions() {
        let mut strategy = valid_strategy();
        strategy.metadata.version = 2;
        assert!(matches!(strategy.validate(), Err(SuZapretError::InvalidStrategy(message)) if message.contains("unsupported format version")));
    }

    #[test]
    fn rejects_multiline_arguments() {
        let mut strategy = valid_strategy();
        strategy.profiles[0].arguments.push("bad\nargument".into());
        assert!(strategy.validate().is_err());
    }

    #[test]
    fn store_lists_and_loads_strategies() {
        let directory = temporary_directory();
        assert!(fs::create_dir_all(&directory).is_ok());
        let strategy_path = directory.join("test.toml");
        let serialized = toml::to_string(&valid_strategy());
        assert!(serialized.is_ok());
        assert!(fs::write(&strategy_path, serialized.unwrap_or_default()).is_ok());

        let store = StrategyStore::new(&directory);
        assert_eq!(store.list().ok(), Some(vec!["test".into()]));
        assert_eq!(store.load("test").ok().map(|strategy| strategy.metadata.id), Some("test".into()));
        assert!(store.load("../test").is_err());
        assert!(fs::remove_dir_all(directory).is_ok());
    }
}
