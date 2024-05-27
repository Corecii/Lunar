use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::Result;

pub fn get_or_init_local_data() -> &'static Path {
    // NOTE: we prefer to keep names short so we don't hit the 256-char path
    // limit on Windows. This typically causes Git to break.

    static DIR: OnceLock<PathBuf> = OnceLock::new();
    let dir_buf = DIR.get_or_init(|| {
        let local_dir = match std::env::var_os("LUNAR_TR_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => dirs::data_local_dir().expect("Could not find local_data directory"),
        };

        // [lunar]-[t]ask[r]unner
        let dir_buf = local_dir.join(".lunar-tr");

        std::fs::create_dir_all(&dir_buf).expect("Could not create local_data directory");
        std::fs::create_dir_all(dir_buf.join("repos"))
            .expect("Could not create local_data/repos directory");

        dir_buf
    });

    dir_buf.as_path()
}

pub fn purge_caches() -> Result<()> {
    let dir = get_or_init_local_data();

    let repo_data_cache = dir.join("repo_data_cache.json");
    let repo_hash_cache = dir.join("repo_hash_cache.json");
    let trust_cache = dir.join("trust_cache.json");

    if repo_data_cache.is_dir() {
        std::fs::remove_dir_all(repo_data_cache)?;
    }
    if repo_hash_cache.is_file() {
        std::fs::remove_file(repo_hash_cache)?;
    }
    if trust_cache.is_file() {
        std::fs::remove_file(trust_cache)?;
    }

    Ok(())
}
