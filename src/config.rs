use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use failure::{bail, ensure, ResultExt};
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
pub struct FileTargetConfiguration {
    pub mode: DeployMode,
    #[serde(default = "default_branch")]
    branch: String,
    build: Option<BuildConfiguration>,
    // upload options
    auth_token: Option<String>,
    username: Option<String>,
    password: Option<String>,
    #[serde(default = "default_hostname")]
    hostname: String,
    #[serde(default)]
    ssl: Option<bool>,
    port: Option<i32>,
    prefix: Option<String>,
    // copy options
    destination: Option<PathBuf>,
    #[serde(default = "default_prune")]
    prune: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CopyConfiguration {
    pub destination: PathBuf,
    pub branch: String,
    pub build: Option<BuildConfiguration>,
    #[serde(default = "default_prune")]
    pub prune: bool,
}

impl CopyConfiguration {
    pub fn new(config: FileTargetConfiguration) -> Result<CopyConfiguration, failure::Error> {
        let FileTargetConfiguration {
            destination,
            branch,
            build,
            prune,
            ..
        } = config;

        let destination = if destination.is_some() {
            destination.unwrap()
        } else {
            bail!("destination must be set for each copy section of the configuration")
        };

        Ok(CopyConfiguration {
            destination,
            branch,
            build,
            prune,
        })
    }
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

#[derive(Clone, Debug)]
pub struct UploadConfiguration {
    pub authentication: Authentication,
    pub hostname: String,
    pub branch: String,
    pub build: Option<BuildConfiguration>,
    pub ssl: bool,
    pub port: i32,
    pub prefix: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Authentication {
    Token(String),
    Basic { username: String, password: String },
}

impl UploadConfiguration {
    pub fn new(config: FileTargetConfiguration) -> Result<UploadConfiguration, failure::Error> {
        let FileTargetConfiguration {
            auth_token,
            username,
            password,
            branch,
            build,
            hostname,
            ssl,
            port,
            prefix,
            ..
        } = config;

        let ssl = ssl.unwrap_or_else(|| hostname == "screeps.com");
        let port = port.unwrap_or_else(|| if ssl { 443 } else { 80 });

        let authentication = if auth_token.is_some() {
            Authentication::Token(auth_token.unwrap())
        } else if username.is_some() && password.is_some() {
            Authentication::Basic {
                username: username.unwrap(),
                password: password.unwrap(),
            }
        } else {
            bail!("either auth_token or username/password must be set in the [upload] section of the configuration");
        };

        Ok(UploadConfiguration {
            authentication,
            branch,
            build,
            hostname,
            ssl,
            port,
            prefix,
        })
    }
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeployMode {
    Copy,
    Upload,
}

#[derive(Clone, Debug, Deserialize)]
struct FileConfiguration {
    default_deploy_target: Option<String>,
    #[serde(default)]
    build: BuildConfiguration,
    targets: HashMap<String, FileTargetConfiguration>,
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub default_deploy_target: Option<String>,
    pub build: BuildConfiguration,
    pub targets: HashMap<String, FileTargetConfiguration>,
}

impl Configuration {
    fn new(config: FileConfiguration) -> Result<Configuration, failure::Error> {
        Ok(Configuration {
            default_deploy_target: config.default_deploy_target,
            build: config.build,
            targets: config.targets,
        })
    }
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

        let file_config: FileConfiguration =
            serde_ignored::deserialize(&mut toml::Deserializer::new(&config_str), |unused_path| {
                unused_paths.insert(unused_path.to_string());
            })
            .context("deserializing config")?;

        for path in &unused_paths {
            warn!("unused configuration path: {}", path)
        }

        Configuration::new(file_config)
    }
}
