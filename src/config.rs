use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use failure::{ensure, ResultExt};
use log::*;
use merge::Merge;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildProfile {
    Dev,
    Profiling,
    Release,
}

#[derive(Clone, Debug, Deserialize, Default, Merge)]
pub struct BuildConfiguration {
    #[serde(default)]
    pub build_profile: Option<BuildProfile>,
    #[serde(default)]
    pub out_name: Option<String>,
    #[merge(strategy = merge::vec::overwrite_empty)]
    #[serde(default)]
    pub extra_options: Vec<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum ModeConfiguration {
    Copy {
        destination: PathBuf,
        #[serde(default = "default_branch")]
        branch: String,
        #[serde(default)]
        build: BuildConfiguration,
        #[serde(default = "default_include_files")]
        include_files: Vec<PathBuf>,
        #[serde(default = "default_prune")]
        prune: bool,
    },
    Upload {
        #[serde(flatten)]
        authentication: Authentication,
        #[serde(default = "default_branch")]
        branch: String,
        #[serde(default)]
        build: BuildConfiguration,
        #[serde(default = "default_hostname")]
        hostname: String,
        #[serde(default = "default_include_files")]
        include_files: Vec<PathBuf>,
        #[serde(default = "default_ssl")]
        ssl: bool,
        #[serde(default = "default_port")]
        port: u16,
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        http_timeout: Option<u32>,
    },
}

fn default_include_files() -> Vec<PathBuf> {
    vec!["pkg".into(), "javascript".into()]
}

fn default_branch() -> String {
    "default".to_owned()
}

fn default_hostname() -> String {
    "screeps.com".to_owned()
}

fn default_prune() -> bool {
    false
}

fn default_ssl() -> bool {
    true
}

fn default_port() -> u16 {
    443
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Authentication {
    Token { auth_token: String },
    Basic { username: String, password: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    pub default_deploy_mode: Option<String>,
    #[serde(default)]
    pub build: BuildConfiguration,
    #[serde(flatten)]
    pub modes: HashMap<String, ModeConfiguration>,
}

impl Configuration {
    pub fn read<P: AsRef<Path>>(config_file: P) -> Result<Self, failure::Error> {
        let config_file = config_file.as_ref();
        ensure!(
            config_file.exists(),
            "expected configuration to exist at {}",
            config_file.display(),
        );

        let config_str = {
            use std::io::Read;
            let mut buf = String::new();
            fs::File::open(config_file)
                .context("opening config file")?
                .read_to_string(&mut buf)
                .context("reading config file")?;
            buf
        };

        let mut unused_paths = BTreeSet::new();

        let config: Configuration =
            serde_ignored::deserialize(&mut toml::Deserializer::new(&config_str), |unused_path| {
                unused_paths.insert(unused_path.to_string());
            })
            .context("deserializing config")?;

        for path in &unused_paths {
            warn!("unused configuration path: {}", path)
        }

        Ok(config)
    }
}
