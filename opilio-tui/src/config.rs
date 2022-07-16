use anyhow::{anyhow, bail, Ok, Result};
use std::{fs, path::PathBuf};

use opilio_lib::Config;
const CONFIG_DIR_NAME: &str = "opilio";
const CONFIG_FILE_NAME: &str = "opilio.toml";

fn config_file() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| anyhow!("Could not find user config directory"))?
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
        let configs: Config = toml::from_str(&data)?;
        Ok(configs)
    } else {
        bail!("Could not find config file")
    }
}

pub fn save_config(config: &Config) -> Result<()> {
    let path = config_file()?;
    let data = toml::to_string_pretty(&config)?;

    fs::write(path, data)?;
    Ok(())
}
