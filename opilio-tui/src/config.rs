use std::{fs, path::PathBuf};

use anyhow::{anyhow, bail, Ok, Result};
use opilio_lib::Config;
const CONFIG_DIR_NAME: &str = "opilio";
const CONFIG_FILE_NAME: &str = "opilio.json";

pub fn config_file() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("User config directory does not exist"))?
        .join(CONFIG_DIR_NAME);

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir.join(CONFIG_FILE_NAME))
}

pub fn from_disk() -> Result<Config> {
    let path = config_file()?;
    if path.exists() {
        let data = fs::read_to_string(path)?;
        let configs: Config = serde_json::from_str(&data)?;
        Ok(configs)
    } else {
        bail!("Config file does not exists")
    }
}
