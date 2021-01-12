use std::{io, path::PathBuf};

use clap::AppSettings;
use failure::format_err;

#[derive(Clone, Debug)]
pub struct CliConfig {
    pub command: Command,
    pub config_path: Option<PathBuf>,
    pub deploy_target: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Check,
    Build,
    Deploy,
}

fn app() -> clap::App<'static, 'static> {
    clap::App::new("cargo screeps")
        .bin_name("cargo")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            clap::SubCommand::with_name("screeps")
                .author("David Ross")
                .version(clap::crate_version!())
                .about("Builds WASM-targetting Rust code and deploys to Screeps game servers")
                .setting(AppSettings::ArgRequiredElseHelp)
                .arg(
                    clap::Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        .multiple(true),
                )
                .arg(
                    clap::Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .multiple(false)
                        .takes_value(true)
                        .value_name("CONFIG_FILE"),
                )
                .subcommand(
                    clap::SubCommand::with_name("build")
                        .about("build files, put in target/ in project root"),
                )
                .subcommand(
                    clap::SubCommand::with_name("check")
                        .about("runs 'cargo check' with appropriate target"),
                )
                .subcommand(
                    clap::SubCommand::with_name("deploy")
                        .about("run target deploy action (or the default if none is specified)")
                        .arg(
                            clap::Arg::with_name("target")
                                .short("t")
                                .long("target")
                                .multiple(false)
                                .takes_value(true)
                                .value_name("DEPLOY_TARGET"),
                        ),
                ),
        )
}

pub fn setup_cli() -> Result<CliConfig, failure::Error> {
    let cargo_args = app().get_matches();

    let args = cargo_args.subcommand_matches("screeps").ok_or_else(|| {
        format_err!("expected first subcommand to be 'screeps'. please run as 'cargo screeps'")
    })?;

    let verbosity = match args.occurrences_of("verbose") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, message, record| out.finish(format_args!("{}: {}", record.target(), message)))
        .chain(io::stdout())
        .apply()
        .unwrap();

    let command = match args.subcommand_name() {
        Some("build") => Command::Build,
        Some("check") => Command::Check,
        Some("deploy") => Command::Deploy,
        other => panic!("unexpected subcommand {:?}", other),
    };
    let config = CliConfig {
        command,
        config_path: args.value_of("config").map(Into::into),
        deploy_target: match args.subcommand_matches("deploy") {
            Some(deploy_args) => deploy_args.value_of("target").map(Into::into),
            None => None,
        },
    };

    Ok(config)
}
