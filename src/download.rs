use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    process::{self, Output},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use base64::Engine;
use regex::Regex;
use uuid::Uuid;

use crate::{
    local_data::get_or_init_local_data, repo_hash::fetch_repo_hash, security::prompt_for_trust,
    task_config::StandaloneTaskConfigRepo,
};

pub trait SuccessOrError {
    fn success_or_error(self) -> Result<Self>
    where
        Self: Sized;
}

impl SuccessOrError for Output {
    fn success_or_error(self) -> Result<Self> {
        if self.status.success() {
            Ok(self)
        } else {
            Err(anyhow::anyhow!(
                "Failed with exit code {}\n{}\n{}",
                self.status.code().unwrap_or(1),
                String::from_utf8_lossy(&self.stderr),
                String::from_utf8_lossy(&self.stdout),
            ))
        }
    }
}

pub fn fetch_repo(info: &StandaloneTaskConfigRepo) -> Result<PathBuf> {
    // We shorten the hash to a more-than-necessary-but-still-shortish 20 chars.
    // We do this to avoid hitting the Windows 256-char path limit, which tends
    // to break Git.

    let hash = fetch_repo_hash(&info)?;
    let short_hash = hash.chars().take(20).collect::<String>();

    static REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = REGEX.get_or_init(|| Regex::new(r"[^a-zA-Z0-9_\+\-\(\)]+").unwrap());

    // We include the sanitized url only for debugging purposes.
    let url_sanitized = regex.replace_all(&info.url, "_");

    // We use a hash of the url to actually differentiate between repos,
    // especially if they have nearly the same name.
    // This technically means repo names are case-sensitive!
    let mut url_hasher = DefaultHasher::new();
    info.url.hash(&mut url_hasher);
    let url_hash =
        base64::prelude::BASE64_STANDARD_NO_PAD.encode(url_hasher.finish().to_le_bytes());

    let repo_cache_dir_name = format!("{}-{}", url_sanitized, url_hash);

    let cache_dir = get_or_init_local_data()
        .join("repos")
        .join(&repo_cache_dir_name)
        .join(&short_hash);

    if cache_dir.exists() {
        return Ok(cache_dir);
    }

    // We use a temporary cache dir while cloning, then move it to the correct
    // spot. This allows multiple clones to be done concurrently. Running
    // multiple clones concurrently this way is technically wasteful, but it's
    // rare and this is a simple solution. The first clone to rename the dir
    // will cause all other clones to use the new dir and delete their own temp
    // clones.
    let cache_dir_temp = cache_dir.with_file_name(format!(
        "{}.tmp_{}",
        short_hash,
        base64::prelude::BASE64_STANDARD_NO_PAD.encode(Uuid::new_v4().to_bytes_le())
    ));
    std::fs::create_dir_all(&cache_dir_temp).context("Could not create cache directory")?;

    process::Command::new("git")
        .current_dir(&cache_dir_temp)
        .arg("init")
        .output()?
        .success_or_error()?;

    process::Command::new("git")
        .current_dir(&cache_dir_temp)
        .args(["remote", "add", "origin", &info.url])
        .output()?
        .success_or_error()?;

    process::Command::new("git")
        .current_dir(&cache_dir_temp)
        .args(["fetch", "--depth", "1", "origin", &hash])
        .output()?
        .success_or_error()?;

    process::Command::new("git")
        .current_dir(&cache_dir_temp)
        .args(["reset", "--hard", "FETCH_HEAD"])
        .output()?
        .success_or_error()?;

    let setup_file = cache_dir_temp.join("lunar-setup.luau");
    if setup_file.exists() {
        let is_trusted = prompt_for_trust(
            &format!("lunar-setup for {}", info.url),
            &format!("Do you want to run the setup script for {} ? It may make changes to your system!\nTemporary file location: {}", info.url, setup_file.display())
        )?;

        if !is_trusted {
            return Err(anyhow::anyhow!("Setup file was not trusted."));
        }

        process::Command::new("lune")
            .current_dir(&cache_dir_temp)
            .arg("run")
            .arg(&setup_file)
            .output()?
            .success_or_error()?;
    }

    match std::fs::rename(&cache_dir_temp, &cache_dir) {
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                match std::fs::remove_dir_all(&cache_dir_temp) {
                    Err(e) => eprintln!(
                        "Failed to remove cache directory (continuing anyways): {:?} {}",
                        &cache_dir_temp, e
                    ),
                    _ => (),
                };
            }
            _ => Err(e).with_context(|| "Failed to rename cache directory")?,
        },
        _ => (),
    };

    Ok(cache_dir)
}
