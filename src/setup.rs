use std::{io, path::PathBuf};

use failure::format_err;

#[derive(Clone, Debug)]
pub struct CliConfig {
    pub command: Command,
    pub config_path: Option<PathBuf>,
    pub deploy_mode: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Build,
    Deploy,
}

fn app() -> clap::Command {
    clap::Command::new("cargo screeps")
        .bin_name("cargo")
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new("screeps")
                .author("David Ross")
                .version(clap::crate_version!())
                .about("Builds WASM-targeting Rust code and deploys to Screeps game servers")
                .arg_required_else_help(true)
                .arg(
                    clap::Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show more information in command output; use multiple times for additional output")
                        .action(clap::ArgAction::Count),
                )
                .arg(
                    clap::Arg::new("config")
                        .short('c')
                        .long("config")
                        .num_args(1)
                        .value_name("CONFIG_FILE"),
                )
                .subcommand(
                    clap::Command::new("build")
                        .about("build files, put in target/ in project root"),
                )
                .subcommand(
                    clap::Command::new("deploy")
                        .about("run specified deploy mode (or the default if none is specified)")
                        .arg(
                            clap::Arg::new("mode")
                                .short('m')
                                .long("mode")
                                .num_args(1)
                                .value_name("DEPLOY_MODE"),
                        ),
                )
                .subcommand(clap::Command::new("copy").about("run the copy deploy mode"))
                .subcommand(
                    clap::Command::new("upload").about("run the upload deploy mode"),
                ),
        )
}

pub fn setup_cli() -> Result<CliConfig, failure::Error> {
    let cargo_args = app().get_matches();

    let args = cargo_args.subcommand_matches("screeps").ok_or_else(|| {
        format_err!("expected first subcommand to be 'screeps'. please run as 'cargo screeps'")
    })?;

    let verbosity = match args.get_count("verbose") {
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

    let mut mode = match args.subcommand_matches("deploy") {
        Some(deploy_args) => deploy_args.get_one::<String>("mode").map(Into::into),
        None => None,
    };

    let command = match args.subcommand_name() {
        Some("build") => Command::Build,
        Some("deploy") => Command::Deploy,
        Some("copy") => {
            mode = Some("copy".to_owned());
            Command::Deploy
        }
        Some("upload") => {
            mode = Some("upload".to_owned());
            Command::Deploy
        }
        other => panic!("unexpected subcommand {other:?}"),
    };
    let config = CliConfig {
        command,
        config_path: args.get_one::<PathBuf>("config").map(Into::into),
        deploy_mode: mode,
    };

    Ok(config)
}
