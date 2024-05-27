use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use cli::{basic_cli_matches, cli};
use task_config::get_directory_tasks;
use task_info::get_task_infos;

mod cli;
mod download;
mod local_data;
mod repo_hash;
mod security;
mod task_config;
mod task_info;

fn main_resulting() -> Result<()> {
    let args = basic_cli_matches();

    if let Ok(args) = args {
        if args.get_flag("clear-cache") {
            local_data::purge_caches()?;
            println!("Repo data cache, repo hash cache, and trust cache cleared.");
            std::process::exit(0);
        };
    };

    let mut tasks: HashMap<String, PathBuf> = HashMap::new();
    get_directory_tasks(&std::env::current_dir().unwrap(), &mut tasks)?;

    let tasks = get_task_infos(&tasks)?;

    let args = cli(&tasks)?.get_matches();

    match args.subcommand() {
        Some((name, args)) => {
            let task = tasks.get(name).ok_or(anyhow::anyhow!("task not found"))?;

            match &task.path {
                None => Err(anyhow::anyhow!("task not runnable"))?,
                Some(script_path) => {
                    let exit_status = std::process::Command::new("lune")
                        .current_dir(std::env::current_dir().unwrap())
                        .args(["run", script_path.to_str().unwrap(), "--"])
                        .args(&task.subtask_args)
                        .args(args.get_many::<String>("args").unwrap_or_default())
                        .stdin(std::process::Stdio::inherit())
                        .stdout(std::process::Stdio::inherit())
                        .stderr(std::process::Stdio::inherit())
                        .spawn()?
                        .wait()?;

                    std::process::exit(exit_status.code().unwrap_or(1));
                }
            }
        }
        None => Err(anyhow::anyhow!("task not found"))?,
    }

    Ok(())
}

fn main() {
    if let Err(e) = main_resulting() {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}
