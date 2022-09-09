use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use failure::{ensure, ResultExt};
use log::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct BuildConfiguration {
    #[serde(default = "BuildConfiguration::default_output_wasm_file")]
    pub output_wasm_file: PathBuf,
    #[serde(default = "BuildConfiguration::default_output_js_file")]
    pub output_js_file: PathBuf,
    #[serde(default)]
    pub initialization_header_file: Option<PathBuf>,
    #[serde(default)]
    pub features: Vec<String>,
}

impl Default for BuildConfiguration {
    fn default() -> Self {
        BuildConfiguration {
            output_wasm_file: Self::default_output_wasm_file(),
            output_js_file: Self::default_output_js_file(),
            initialization_header_file: None,
            features: vec![],
        }
    }
}

impl BuildConfiguration {
    fn default_output_js_file() -> PathBuf {
        "main.js".into()
    }
    fn default_output_wasm_file() -> PathBuf {
        "compiled.wasm".into()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BuildOverrides {
    #[serde(default)]
    pub output_wasm_file: Option<PathBuf>,
    #[serde(default)]
    pub output_js_file: Option<PathBuf>,
    #[serde(default)]
    pub initialization_header_file: Option<PathBuf>,
    #[serde(default)]
    pub features: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum ModeConfiguration {
    Copy {
        destination: PathBuf,
        #[serde(default = "default_branch")]
        branch: String,
        #[serde(default)]
        build: Option<BuildOverrides>,
        #[serde(default = "default_prune")]
        prune: bool,
    },
    Upload {
        #[serde(flatten)]
        authentication: Authentication,
        #[serde(default = "default_branch")]
        branch: String,
        #[serde(default)]
        build: Option<BuildOverrides>,
        #[serde(default = "default_hostname")]
        hostname: String,
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
