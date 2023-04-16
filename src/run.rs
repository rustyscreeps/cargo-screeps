use std::path::{Path, PathBuf};

use anyhow::anyhow;
use log::*;
use merge::Merge;

use crate::{
    build,
    config::{self, Authentication, BuildConfiguration, ModeConfiguration},
    copy, orientation, setup, upload,
};

pub fn run() -> Result<(), anyhow::Error> {
    let cli_config = setup::setup_cli()?;

    let root = orientation::find_project_root(&cli_config)?;
    let config_path = cli_config
        .config_path
        .unwrap_or_else(|| root.join("screeps.toml"));

    let mut config = config::Configuration::read(&config_path)?;

    debug!(
        "Running {:?} at {:?} using config {:?} with values {:#?}",
        cli_config.command, root, config_path, config
    );

    match cli_config.command {
        setup::Command::Build => run_build(&root, &config.build)?,
        setup::Command::Deploy => {
            let mode = match cli_config.deploy_mode {
                Some(v) => v,
                None => {
                    config.default_deploy_mode.ok_or_else(|| {
                        anyhow!("must have default_deploy_mode set to use 'cargo screeps deploy' without --mode")
                    })?
                }
            };
            let target_config = config.modes.remove(&mode).ok_or_else(|| {
                anyhow!(
                    "couldn't find mode {}, must be defined in screeps.toml",
                    mode
                )
            })?;
            match target_config {
                ModeConfiguration::Copy {
                    destination,
                    branch,
                    build,
                    include_files,
                    prune,
                } => {
                    config.build.merge(build);
                    run_build(&root, &config.build)?;
                    run_copy(
                        &root,
                        &config.build.path,
                        &destination,
                        &branch,
                        &include_files,
                        prune,
                    )?;
                }
                ModeConfiguration::Upload {
                    authentication,
                    branch,
                    build,
                    include_files,
                    hostname,
                    ssl,
                    port,
                    prefix,
                    http_timeout,
                } => {
                    let url = format!(
                        "{}://{}:{}/{}",
                        if ssl { "https" } else { "http" },
                        hostname,
                        port,
                        match prefix {
                            Some(prefix) => format!("{prefix}/api/user/code"),
                            None => "api/user/code".to_string(),
                        }
                    );

                    config.build.merge(build);
                    run_build(&root, &config.build)?;
                    run_upload(
                        &root,
                        &config.build.path,
                        &authentication,
                        &branch,
                        &include_files,
                        &url,
                        http_timeout,
                    )?;
                }
            };
        }
    }

    Ok(())
}

fn run_build(root: &Path, config: &BuildConfiguration) -> Result<(), anyhow::Error> {
    info!("compiling...");
    build::build(root, config)?;
    info!("compiled.");

    Ok(())
}

fn run_copy(
    root: &Path,
    build_path: &Option<PathBuf>,
    destination: &PathBuf,
    branch: &String,
    include_files: &Vec<PathBuf>,
    prune: bool,
) -> Result<(), anyhow::Error> {
    info!("copying...");
    copy::copy(root, build_path, destination, branch, include_files, prune)?;
    info!("copied.");

    Ok(())
}

fn run_upload(
    root: &Path,
    build_path: &Option<PathBuf>,
    authentication: &Authentication,
    branch: &String,
    include_files: &Vec<PathBuf>,
    url: &String,
    http_timeout: Option<u32>,
) -> Result<(), anyhow::Error> {
    info!("uploading...");
    upload::upload(
        root,
        build_path,
        authentication,
        branch,
        include_files,
        url,
        http_timeout,
    )?;
    info!("uploaded.");

    Ok(())
}
