use std::path::{Path, PathBuf};

use failure::format_err;
use log::*;

use crate::{
    build,
    config::{self, Authentication, BuildConfiguration, BuildOverrides, ModeConfiguration},
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
                        prune,
                    } => {
                        if build.is_some() {
                            override_build_options(&mut config.build, build.unwrap());
                        }
                        run_build(&root, &config.build)?;
                        run_copy(
                            &root,
                            &destination,
                            &branch,
                            prune,
                            &config.build.output_js_file,
                            &config.build.output_wasm_file,
                        )?;
                    }
                    ModeConfiguration::Upload {
                        authentication,
                        branch,
                        build,
                        hostname,
                        ssl,
                        port,
                        prefix,
                        http_timeout,
                    } => {
                        if build.is_some() {
                            override_build_options(&mut config.build, build.unwrap());
                        }
                        run_build(&root, &config.build)?;
                        run_upload(
                            &root,
                            &authentication,
                            &branch,
                            &hostname,
                            ssl,
                            port,
                            &prefix,
                            http_timeout,
                        )?;
                    }
                },
            }
        }
    }

    Ok(())
}

fn override_build_options(build_config: &mut BuildConfiguration, overrides: BuildOverrides) {
    let BuildOverrides {
        output_wasm_file,
        output_js_file,
        initialization_header_file,
        features,
    } = overrides;

    if output_wasm_file.is_some() {
        build_config.output_wasm_file = output_wasm_file.unwrap();
    }

    if output_js_file.is_some() {
        build_config.output_js_file = output_js_file.unwrap();
    }

    if initialization_header_file.is_some() {
        build_config.initialization_header_file = initialization_header_file;
    }

    if features.is_some() {
        build_config.features = features.unwrap();
    }
}

fn run_build(root: &Path, config: &BuildConfiguration) -> Result<(), failure::Error> {
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
    destination: &PathBuf,
    branch: &String,
    prune: bool,
    output_js_file: &PathBuf,
    output_wasm_file: &PathBuf,
) -> Result<(), failure::Error> {
    info!("copying...");
    copy::copy(
        root,
        destination,
        branch,
        prune,
        output_js_file,
        output_wasm_file,
    )?;
    info!("copied.");

    Ok(())
}

fn run_upload(
    root: &Path,
    authentication: &Authentication,
    branch: &String,
    hostname: &String,
    ssl: bool,
    port: u16,
    prefix: &Option<String>,
    http_timeout: Option<u32>,
) -> Result<(), failure::Error> {
    info!("uploading...");
    upload::upload(
        root,
        authentication,
        branch,
        hostname,
        ssl,
        port,
        prefix,
        http_timeout,
    )?;
    info!("uploaded.");

    Ok(())
}
