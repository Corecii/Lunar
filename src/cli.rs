use std::{collections::HashMap, sync::OnceLock};

use anyhow::Result;
use clap::{Arg, Command};

use crate::task_info::TaskInfo;

pub fn cli(tasks: &HashMap<String, TaskInfo>) -> Result<Command> {
    let mut command = Command::new("lunar")
        .about("Lunar: Task Runner for Lune")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("trust-new")
                .long("trust-new")
                .short('y')
                .help("Trust new repos by default")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("clear-cache")
                .long("clear-cache")
                .help("Clear repo data cache, repo hash cache, and trust cache")
                .action(clap::ArgAction::SetTrue),
        );

    let mut task_entries = tasks.iter().collect::<Vec<_>>();
    task_entries.sort_by_key(|x| x.0);

    for (name, task) in task_entries {
        let mut subcommand = Command::new(name)
            .arg(
                Arg::new("args")
                    .num_args(0..)
                    .trailing_var_arg(true)
                    .allow_hyphen_values(true),
            )
            .disable_help_flag(true);

        if let Some(args) = task.args.as_ref() {
            subcommand = subcommand.override_usage(format!("{} {}", &name, args));
        } else {
            subcommand = subcommand.override_usage(name);
        }
        if let Some(about) = task.about.as_ref() {
            subcommand = subcommand.about(about);
        }

        subcommand = subcommand.help_template(
            "\
{before-help}{about-with-newline}
{usage-heading} {usage}{after-help}
",
        );

        command = command.subcommand(subcommand);
    }

    Ok(command)
}

pub fn basic_cli() -> &'static Command {
    static BASIC_CLI: OnceLock<Command> = OnceLock::new();

    BASIC_CLI.get_or_init(|| {
        cli(&HashMap::new())
            .unwrap()
            .allow_external_subcommands(true)
            .arg_required_else_help(false)
            .subcommand_required(false)
            .disable_colored_help(true)
    })
}

pub fn basic_cli_matches() -> &'static Result<clap::ArgMatches, clap::error::Error> {
    static MATCHES: OnceLock<Result<clap::ArgMatches, clap::error::Error>> = OnceLock::new();
    MATCHES.get_or_init(|| basic_cli().clone().try_get_matches())
}
