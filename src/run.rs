use std::path::{Path, PathBuf};

use failure::format_err;
use log::*;
use merge::Merge;

use crate::{
    build,
    config::{self, Authentication, BuildConfiguration, ModeConfiguration},
    copy, orientation, setup, upload,
};

pub fn run() -> Result<(), failure::Error> {
    let cli_config = setup::setup_cli()?;

    let root = orientation::find_project_root(&cli_config)?;
    let config_path = cli_config
        .config_path
        .unwrap_or_else(|| root.join("screeps.toml").to_owned());

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
                        format_err!("must have default_deploy_mode set to use 'cargo screeps deploy' without --mode")
                    })?
                }
            };

            match config.modes.remove(&mode).ok_or_else(|| {
                format_err!(
                    "couldn't find mode {}, must be defined in screeps.toml",
                    mode
                )
            })? {
                target_config => match target_config {
                    ModeConfiguration::Copy {
                        destination,
                        branch,
                        build,
                        include_files,
                        prune,
                    } => {
                        config.build.merge(build);
                        run_build(&root, &config.build)?;
                        run_copy(&root, &config.build.path, &destination, &branch, &include_files, prune)?;
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
                    } => {
                        config.build.merge(build);
                        run_build(&root, &config.build)?;
                        run_upload(
                            &root,
                            &config.build.path,
                            &authentication,
                            &branch,
                            &include_files,
                            &hostname,
                            ssl,
                            port,
                            &prefix,
                        )?;
                    }
                },
            }
        }
    }

    Ok(())
}

fn run_build(root: &Path, config: &BuildConfiguration) -> Result<(), failure::Error> {
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
) -> Result<(), failure::Error> {
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
    hostname: &String,
    ssl: bool,
    port: u16,
    prefix: &Option<String>,
) -> Result<(), failure::Error> {
    info!("uploading...");
    upload::upload(
        root,
        build_path,
        authentication,
        branch,
        include_files,
        hostname,
        ssl,
        port,
        prefix,
    )?;
    info!("uploaded.");

    Ok(())
}
