use std::{env, io};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::time::Duration;
use mac_address::get_mac_address;

// put under option
use serde_derive::{Serialize, Deserialize};

const CONFIG_FILE_NAME: &str = "wimon.toml";

#[derive(Serialize, Deserialize, Clone)]
enum DeviceId {
    MAC([u8;6])
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum MonitorSpec {
    /// Report status of all SSIDs that are detected at each monitoring moment
    All,
    /// Only report on the status of the connection (wifi or ethernet) used to send results
    Connection,
    /// Monitor a specific list of supplied SSIDs by name
    SSIDs(Vec<String>)
}

impl Default for MonitorSpec {
    fn default() -> Self {
        MonitorSpec::Connection
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Config {
    monitor_spec: MonitorSpec
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceId::MAC(mac) => write!(f, "MAC({:?})", mac)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Stats {
    power_percent: u8
}

#[derive(Serialize, Deserialize)]
struct SSIDReport {
    ssid: String,
    stats: Option<Stats>,
}

#[derive(Serialize, Deserialize)]
struct MonitorReport {
    device_id: DeviceId,
    local_time: Option<String>,
    ssids: Vec<SSIDReport>,
}

impl Display for MonitorReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tDeviceId = {}", self.device_id)
    }
}

fn main() -> Result<(), io::Error> {
    let config_file_path = find_config_file(CONFIG_FILE_NAME)?;
    let config = read_config(&config_file_path)?;
    println!("Config file loaded from: \"{}\"", config_file_path.display());
    println!("Monitor: {:?}", config.monitor_spec);
    let device_id = get_device_id()?;

    monitor_loop(config, device_id)?;

    Ok(())
}

fn monitor_loop(_config: Config, device_id: DeviceId) -> Result<(), io::Error> {
    loop {
        let report = MonitorReport {
            device_id: device_id.clone(),
            local_time: None,
            ssids: vec![]
        };

        println!("Report: \n{report}");

        std::thread::sleep(Duration::from_secs(10));
    }

    //Ok(())
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

fn read_config(config_file_path: &PathBuf) -> Result<Config, io::Error> {
    let config_string = std::fs::read_to_string(config_file_path)?;
    let config : Config = toml::from_str(&config_string).map_err(|_|
        io::Error::new(io::ErrorKind::NotFound,
                       "Could not parse toml config file"))?;
    Ok(config)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::{Config, CONFIG_FILE_NAME, MonitorSpec};

    #[test]
    fn default_config_toml() {
        let config = Config::default();
        assert_eq!(toml::to_string(&config).unwrap(), "monitor_spec = \"Connection\"\n");
    }

    #[test]
    fn bundled_spec() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_dir = manifest_dir.parent().ok_or("Could not get parent dir")
            .expect("Could not get parent dir");
        let config_string = std::fs::read_to_string(root_dir.join(CONFIG_FILE_NAME))
            .unwrap();
        let config: Config = toml::from_str(&config_string).unwrap();
        assert_eq!(config.monitor_spec, MonitorSpec::Connection);
    }

    #[test]
    fn config_all() {
        let config: Config = toml::from_str("monitor_spec=\"All\"\n").unwrap();
        assert_eq!(config.monitor_spec, MonitorSpec::All)
    }

    #[test]
    fn config_ssids() {
        let config: Config = toml::from_str("[monitor_spec]\nSSIDs=['ABC', 'DEF']\n").unwrap();
        let ssid_list = MonitorSpec::SSIDs(vec!["ABC".to_owned(), "DEF".to_owned()]);
        assert_eq!(config.monitor_spec, ssid_list)
    }
}