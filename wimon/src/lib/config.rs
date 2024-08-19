use std::{env, io};
use std::path::PathBuf;
use std::time::Duration;

use serde_derive::{Deserialize, Serialize};
use url::Url;

#[cfg_attr(
    not(feature = "pico"),
    derive(Default, Serialize, Deserialize, Debug, PartialEq)
)]
pub enum MonitorSpec {
    /// Report status of all SSIDs that are detected at each monitoring moment
    All,
    /// Only report on the status of the connection (wifi or ethernet) used to send results
    #[default]
    Connection,
}

#[cfg_attr(
    not(feature = "pico"),
    derive(Serialize, Deserialize, Debug, PartialEq)
)]
pub struct ReportSpec {
    pub period_seconds: Option<u64>,
    pub base_url: Option<String>,
}

#[cfg_attr(
    not(feature = "pico"),
    derive(Default, Serialize, Deserialize)
)]
pub struct Config {
    pub monitor: Option<MonitorSpec>,
    pub report: Option<ReportSpec>,
    #[serde(skip)]
    pub period_duration: Duration,
    #[serde(skip)]
    pub report_url: Option<Url>,
}

pub fn find_config_file(file_name: &str) -> Result<PathBuf, io::Error> {
    let mut dir = env::current_dir().ok();

    // Loop until no parent director exists. (i.e. stop at "/")
    while let Some(directory) = dir {
        let config_path = directory.join(file_name);

        if config_path.exists() {
            return Ok(config_path);
        }

        dir = directory.parent().map(|p| p.to_path_buf());
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "wimon toml config file not found",
    ))
}

pub fn read_config(config_file_path: &PathBuf) -> Result<Config, io::Error> {
    let config_string = std::fs::read_to_string(config_file_path)?;
    let mut config: Config = toml::from_str(&config_string)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not parse toml config file"))?;

    match &config.report {
        Some(spec) => {
            config.period_duration = match spec.period_seconds {
                None => Duration::from_secs(60),
                Some(period) => Duration::from_secs(period),
            }
        }
        None => config.period_duration = Duration::from_secs(60),
    }

    config.report_url = match &config.report {
        Some(spec) => match &spec.base_url {
            Some(url_string) => Url::parse(url_string).ok(),
            None => None,
        },
        None => None,
    };

    Ok(config)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SsidSpec {
    pub ssid_name: String,
    pub ssid_pass: String,
}

impl Default for SsidSpec {
    fn default() -> Self {
        SsidSpec {
            ssid_name: "".to_string(),
            ssid_pass: "".to_string(),
        }
    }
}

pub fn read_ssid(ssid_file_path: &PathBuf) -> Result<SsidSpec, io::Error> {
    let ssid_string = std::fs::read_to_string(ssid_file_path)?;
    toml::from_str(&ssid_string)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not parse toml ssid file"))
}

#[cfg(test)]
mod test {
    use super::{Config, MonitorSpec};

    #[test]
    fn config_monitor_connection() {
        let config: Config = toml::from_str("monitor = \"Connection\"\n").unwrap();
        assert_eq!(config.monitor, Some(MonitorSpec::Connection));
    }

    #[test]
    fn config_monitor_all() {
        let config: Config = toml::from_str("monitor=\"All\"\n").unwrap();
        assert_eq!(config.monitor, Some(MonitorSpec::All))
    }

    #[test]
    fn config_with_report_spec() {
        let config: Config = toml::from_str("[report]\nperiod_seconds = 1\n").unwrap();
        assert_eq!(config.report.unwrap().period_seconds, Some(1));
    }
}
