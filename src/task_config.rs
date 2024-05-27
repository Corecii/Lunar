use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use anyhow::Result;
use serde::Deserialize;

use crate::{download::fetch_repo, security::prompt_for_trust};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ScriptSelector {
    pub script: Option<String>,

    pub tasks: Option<Vec<String>>,
    pub prefix: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StandaloneTaskConfigRepo {
    pub url: String,
    pub tag: Option<String>,
    pub hash: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none", flatten)]
    pub selector: Option<ScriptSelector>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StandaloneTaskConfigDirectory {
    pub path: String,

    #[serde(default, skip_serializing_if = "Option::is_none", flatten)]
    pub selector: Option<ScriptSelector>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StandaloneTaskConfig {
    pub name: Option<String>,

    pub repo: Option<StandaloneTaskConfigRepo>,
    pub directory: Option<StandaloneTaskConfigDirectory>,

    pub script: Option<String>,
}

static SCRIPT_DIRS: [&str; 4] = ["lune", ".lune", "lunar", ".lunar"];
static SCRIPT_SINGLE_DIRS: [&str; 5] = ["lune", ".lune", "lunar", ".lunar", "."];

pub fn get_directory_tasks(directory: &Path, tasks: &mut HashMap<String, PathBuf>) -> Result<()> {
    for script_dir in SCRIPT_DIRS {
        let path = directory.join(script_dir);
        if path.exists() {
            for entry in path.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                if path.extension() == Some("lua".as_ref())
                    || path.extension() == Some("luau".as_ref())
                {
                    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                    tasks.insert(name, path);
                } else if path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .ends_with(".lunar.toml")
                {
                    let name = path.file_name().unwrap().to_string_lossy();
                    let name = name.trim_end_matches(".lunar.toml");

                    let config: StandaloneTaskConfig =
                        toml::from_str(&std::fs::read_to_string(&path)?)?;

                    get_standalone_config_tasks(&config, name, tasks)?;
                }
            }
        }
    }

    Ok(())
}

pub fn get_selector_tasks(
    directory: &Path,
    name: &str,
    selector: &ScriptSelector,
    tasks: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    if let Some(script) = &selector.script {
        for script_dir in SCRIPT_SINGLE_DIRS {
            let path = directory.join(script_dir).join(script);
            if path.exists() {
                tasks.insert(name.to_string(), path);
                break;
            }
            let path = directory.join(script_dir).join(format!("{}.luau", script));
            if path.exists() {
                tasks.insert(name.to_string(), path);
                break;
            }
            let path = directory.join(script_dir).join(format!("{}.lua", script));
            if path.exists() {
                tasks.insert(name.to_string(), path);
                break;
            }
        }
    } else {
        let mut subtasks = HashMap::new();
        get_directory_tasks(directory, &mut subtasks)?;

        let scripts_filter: Option<HashSet<String>> = selector
            .tasks
            .clone()
            .map(|s| HashSet::from_iter(s.into_iter()));

        for (name, path) in subtasks {
            if let Some(scripts) = &scripts_filter {
                if !scripts.contains(&name) {
                    continue;
                }
            }

            let name = selector
                .prefix
                .as_ref()
                .map_or_else(|| name.clone(), |prefix| format!("{}{}", prefix, name));

            tasks.insert(name, path);
        }
    }

    Ok(())
}

pub fn get_standalone_config_tasks(
    config: &StandaloneTaskConfig,
    name: &str,
    tasks: &mut HashMap<String, PathBuf>,
) -> Result<()> {
    let name = config.name.as_deref().unwrap_or(name);

    if let Some(script) = &config.script {
        tasks.insert(name.to_string(), PathBuf::from(script));
    }
    if let Some(subconfig) = &config.repo {
        if prompt_for_trust(
            &subconfig.url,
            &format!("Do you want to download {} ?", &subconfig.url),
        )? {
            match fetch_repo(subconfig) {
                Ok(directory) => {
                    get_selector_tasks(
                        &directory,
                        name,
                        &subconfig.selector.clone().unwrap_or_default(),
                        tasks,
                    )?;
                }
                Err(e) => {
                    eprintln!("Skipping tasks from {} because of error:\n{}", &name, e);
                }
            }
        } else {
            eprintln!("Skipping tasks from {} because it's not trusted.\n", &name);
        }
    }
    if let Some(subconfig) = &config.directory {
        let directory = PathBuf::from(&subconfig.path);

        get_selector_tasks(
            &directory,
            name,
            &subconfig.selector.clone().unwrap_or_default(),
            tasks,
        )?;
    }

    Ok(())
}
