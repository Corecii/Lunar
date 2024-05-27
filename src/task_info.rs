use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Result;
use regex::bytes::Regex as RegexBytes;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct InlineTaskInfo {
    pub args: Option<String>,
    pub about: Option<String>,
    pub hide: Option<bool>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tasks: HashMap<String, InlineTaskInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskInfo {
    pub args: Option<String>,
    pub about: Option<String>,

    pub path: Option<PathBuf>,
    pub subtask_args: Vec<String>,
}

fn add_inline_task(
    tasks: &mut HashMap<String, TaskInfo>,
    name: &str,
    path: Option<&Path>,
    info: InlineTaskInfo,
    subtask_args: Vec<String>,
) {
    #[allow(clippy::bool_comparison)]
    if info.hide.unwrap_or(false) == false {
        tasks.insert(
            name.to_string(),
            TaskInfo {
                args: info.args,
                about: info.about,
                path: path.map(PathBuf::from),
                subtask_args: subtask_args.clone(),
            },
        );
    }

    if !info.tasks.is_empty() {
        for (subname, subtask) in info.tasks {
            let mut subsubtask_args = subtask_args.clone();
            subsubtask_args.push(subname.clone());

            add_inline_task(tasks, &subname, path, subtask, subsubtask_args);
        }
    }
}

pub fn get_task_infos(tasks: &HashMap<String, PathBuf>) -> Result<HashMap<String, TaskInfo>> {
    let mut infos = HashMap::new();
    for (name, task) in tasks {
        let info = get_base_script_info(task)?;

        add_inline_task(&mut infos, name, Some(task), info, vec![]);
    }

    Ok(infos)
}

fn get_base_script_info(script_path: &Path) -> Result<InlineTaskInfo> {
    let contents = std::fs::read(script_path)?;

    static REGEX: OnceLock<RegexBytes> = OnceLock::new();
    let regex = REGEX.get_or_init(|| {
        RegexBytes::new(r#"(?s)--\[=\[[^\S\r\n]*lunar[^\S\n]*\n(.*?)\-*]=\]"#).unwrap()
    });
    // Matches:
    // --[=[ lunar
    //     ...
    // --]=]
    // (where the final -- is optional, and spaces before and after "lunar" are ignored)

    let captures = regex.captures(&contents);
    if captures.is_none() {
        return Ok(InlineTaskInfo {
            args: None,
            about: None,
            hide: None,
            tasks: HashMap::new(),
        });
    }
    let captures = captures.unwrap();

    let info = String::from_utf8_lossy(captures.get(1).unwrap().as_bytes());
    match toml::from_str(&info) {
        Ok(info) => Ok(info),
        Err(e) => Ok(InlineTaskInfo {
            args: None,
            about: Some(format!("[failed to parse info: {}]", e.message())),
            hide: None,
            tasks: HashMap::new(),
        }),
    }
}
