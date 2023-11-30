use std::{env, io};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use mac_address::get_mac_address;

enum DeviceId {
    MAC([u8;6])
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceId::MAC(mac) => writeln!(f, "MAC({:?})", mac)
        }
    }
}
const CONFIG_FILE_NAME: &str = "wimon.toml";

fn main() -> Result<(), io::Error> {
    let config_file_path = find_config_file(CONFIG_FILE_NAME)?;
    println!("Config file found at {}", config_file_path.display());

    let device_id = get_device_id()?;
    println!("Device ID = {device_id}");

    Ok(())
}

fn get_device_id() -> Result<DeviceId, io::Error> {
    match get_mac_address() {
        Ok(Some(ma)) => Ok(DeviceId::MAC(ma.bytes())),
        _ => Err(io::Error::new(io::ErrorKind::NotFound, "DeviceId could not be determined"))
    }
}
fn find_config_file(file_name: &str) -> Result<PathBuf, io::Error> {
    let mut dir = env::current_dir().ok();

    // Loop until no parent director exists. (i.e. stop at "/")
    while let Some(directory) = dir {
        let config_path = directory.join(file_name);

        if config_path.exists() {
            return Ok(config_path);
        }

        dir = directory.parent().map(|p| p.to_path_buf());
    }

    Err(io::Error::new(io::ErrorKind::NotFound,
                        "wimon toml config file not found"))
}