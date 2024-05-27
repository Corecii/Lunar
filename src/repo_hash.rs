use std::{collections::HashMap, sync::OnceLock};

use anyhow::{anyhow, Context, Result};
use regex::bytes::Regex as RegexBytes;
use serde::{Deserialize, Serialize};
use std::process;

use crate::{
    download::SuccessOrError, local_data::get_or_init_local_data,
    task_config::StandaloneTaskConfigRepo,
};

#[derive(Default, Debug, Serialize, Deserialize)]
struct RepoHashCache {
    repos: HashMap<String, String>,
}

fn try_read_hash_cache() -> Result<Option<RepoHashCache>> {
    let cache_file = get_or_init_local_data().join("repo_hash_cache.json");
    let contents = match std::fs::read_to_string(cache_file) {
        Ok(contents) => contents,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            _ => {
                return Err(anyhow!(e));
            }
        },
    };

    Ok(serde_json::from_str::<RepoHashCache>(&contents).ok())
}

fn try_write_hash_cache(cache: &RepoHashCache) -> Result<()> {
    let cache_file = get_or_init_local_data().join("repo_hash_cache.json");
    serde_json::to_writer(&std::fs::File::create(cache_file)?, cache)?;
    Ok(())
}

pub fn fetch_repo_hash_cached(info: &StandaloneTaskConfigRepo) -> Result<Option<String>> {
    let cache = try_read_hash_cache()?.unwrap_or_default();

    let repo_key = format!("{} on {}", info.tag.as_deref().unwrap_or("HEAD"), info.url);

    Ok(cache.repos.get(&repo_key).cloned())
}

pub fn add_one_to_hash_cache(info: &StandaloneTaskConfigRepo, hash: &str) -> Result<()> {
    let mut cache = try_read_hash_cache()?.unwrap_or_default();

    let repo_key = format!("{} on {}", info.tag.as_deref().unwrap_or("HEAD"), info.url);

    cache.repos.insert(repo_key, hash.to_string());

    try_write_hash_cache(&cache)?;
    Ok(())
}

pub fn fetch_repo_hash_online(info: &StandaloneTaskConfigRepo) -> Result<String> {
    static REGEX: OnceLock<RegexBytes> = OnceLock::new();
    let regex = REGEX.get_or_init(|| RegexBytes::new(r"^\s*([a-f0-9]{40}).*\s*$").unwrap());

    if info.hash.is_some() {
        if !regex.is_match(info.hash.as_ref().unwrap().as_bytes()) {
            return Err(anyhow::anyhow!(
                "Invalid hash: {}; expected full 40-character hash",
                info.hash.as_ref().unwrap()
            ));
        }

        return Ok(info.hash.clone().unwrap().to_string());
    }

    let tag = info.tag.as_deref().unwrap_or("HEAD");

    let output = process::Command::new("git")
        .arg("ls-remote")
        .arg(&info.url)
        .arg(tag)
        .output()?
        .success_or_error()?;

    let hash_bytes = regex
        .captures(output.stdout.as_slice())
        .context(format!(
            "Failed to fetch repo hash online;\nno results for 'git ls-remote {} {}'",
            &info.url, tag
        ))?
        .get(1)
        .context("Failed to fetch repo hash online (???)")?
        .as_bytes();

    Ok(String::from_utf8_lossy(hash_bytes).to_string())
}

pub fn fetch_repo_hash(info: &StandaloneTaskConfigRepo) -> Result<String> {
    let online_hash = fetch_repo_hash_online(info);
    let cached_hash = fetch_repo_hash_cached(info)?;

    if let Ok(hash) = online_hash {
        if Some(&hash) != cached_hash.as_ref() {
            if let Err(e) = add_one_to_hash_cache(info, &hash) {
                eprintln!("Failed to update hash cache (continuing anyways): {}", e);
            }
        }

        return Ok(hash);
    }

    if let Some(hash) = cached_hash {
        return Ok(hash);
    }

    online_hash.with_context(|| "Could not fetch hash online or from local cache")
}
