use std::path::Path;

use failure::format_err;
use log::*;

use crate::{
    build,
    config::{self, Configuration},
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
        setup::Command::Check => run_check(&root)?,
        setup::Command::Deploy => {
            let target = match cli_config.deploy_target.clone() {
                Some(v) => v,
                None => {
                    config.default_deploy_target.clone().ok_or_else(|| {
                        format_err!("must have default_deploy_target set to use 'cargo screeps deploy' without --target")
                    })?
                }
            };

            match config.targets.remove(&target).ok_or_else(|| {
                format_err!(
                    "couldn't find target {}, must be defined in screeps.toml",
                    target
                )
            })? {
                target_config => match target_config.mode {
                    config::DeployMode::Upload => {
                        let upload_config = config::UploadConfiguration::new(target_config)?;
                        match &upload_config.build {
                            Some(build_config) => run_build(&root, build_config)?,
                            None => run_build(&root, &config.build)?,
                        }
                        run_upload(&root, &upload_config)?
                    }
                    config::DeployMode::Copy => {
                        let copy_config = config::CopyConfiguration::new(target_config)?;
                        match &copy_config.build {
                            Some(build_config) => run_build(&root, build_config)?,
                            None => run_build(&root, &config.build)?,
                        }
                        run_copy(&root, &config, &copy_config)?
                    }
                },
            }
        }
    }

    Ok(())
}

fn run_build(root: &Path, config: &config::BuildConfiguration) -> Result<(), failure::Error> {
    info!("compiling...");
    build::build(root, config)?;
    info!("compiled.");

    Ok(())
}

fn run_check(root: &Path) -> Result<(), failure::Error> {
    info!("checking...");
    build::check(root)?;
    info!("checked.");

    Ok(())
}

fn run_copy(
    root: &Path,
    config: &Configuration,
    copy_config: &config::CopyConfiguration,
) -> Result<(), failure::Error> {
    info!("copying...");
    copy::copy(root, config, copy_config)?;
    info!("copied.");

    Ok(())
}

fn run_upload(
    root: &Path,
    upload_config: &config::UploadConfiguration,
) -> Result<(), failure::Error> {
    info!("uploading...");
    upload::upload(root, upload_config)?;
    info!("uploaded.");

    Ok(())
}
