use std::{collections::HashSet, io::Write};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{cli::basic_cli_matches, local_data::get_or_init_local_data};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
struct TrustFile {
    trusted: HashSet<String>,
}

fn read_trust_file() -> Result<TrustFile> {
    let path = get_or_init_local_data().join("trust_cache.json");

    if !path.exists() {
        return Ok(TrustFile::default());
    }

    Ok(serde_json::from_str(&std::fs::read_to_string(&path)?).unwrap_or_default())
}

fn write_trust_file(trust_file: &TrustFile) -> Result<()> {
    let path = get_or_init_local_data().join("trust_cache.json");
    std::fs::write(&path, serde_json::to_string(trust_file)?)?;
    Ok(())
}

pub fn is_trusted(name: &str) -> bool {
    let trust_file = read_trust_file().unwrap_or_default();
    trust_file.trusted.contains(name)
}

pub fn trust(name: &str) -> Result<()> {
    let mut trust_file = read_trust_file()?;
    trust_file.trusted.insert(name.to_string());
    write_trust_file(&trust_file)?;
    Ok(())
}

pub fn prompt_for_trust(name: &str, query: &str) -> Result<bool> {
    if is_trusted(name) {
        return Ok(true);
    }

    if let Ok(args) = basic_cli_matches() {
        if args.get_flag("trust-new") {
            trust(name)?;
            return Ok(true);
        }
    }

    let mut input = String::new();
    eprintln!("{}", query);
    eprint!("Trust? (y/N) ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        trust(name)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
