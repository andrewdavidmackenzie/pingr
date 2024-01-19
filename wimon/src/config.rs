use std::{env, io};
use std::path::PathBuf;
use std::time::Duration;
use serde_derive::{Deserialize, Serialize};
use url::Url;

pub(crate) const CONFIG_FILE_NAME: &str = "wimon.toml";

#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub(crate) enum MonitorSpec {
    /// Report status of all SSIDs that are detected at each monitoring moment
    All,
    /// Only report on the status of the connection (wifi or ethernet) used to send results
    #[default]
    Connection,
    /// Monitor a specific list of supplied SSIDs by name
    SSIDs(Vec<String>)
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct ReportSpec {
    period_seconds: Option<u64>,
    #[serde(rename="base_url")]
    report_url: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(rename="monitor")]
    pub monitor_spec: Option<MonitorSpec>,
    #[serde(rename="report")]
    pub report_spec: Option<ReportSpec>,
    #[serde(skip)]
    pub period_duration: Duration,
    #[serde(skip)]
    pub report_url: Option<Url>,
}

pub(crate) fn find_config_file(file_name: &str) -> Result<PathBuf, io::Error> {
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

pub(crate) fn read_config(config_file_path: &PathBuf) -> Result<Config, io::Error> {
    let config_string = std::fs::read_to_string(config_file_path)?;
    let mut config : Config = toml::from_str(&config_string).map_err(|_|
        io::Error::new(io::ErrorKind::NotFound,
                       "Could not parse toml config file"))?;

    match &config.report_spec {
        Some(spec) => {
            config.period_duration = match spec.period_seconds {
                None => Duration::from_secs(60),
                Some(period) => Duration::from_secs(period),
            }
        },
        None => config.period_duration = Duration::from_secs(60)
    }

    config.report_url = match &config.report_spec {
        Some(spec) => {
            match &spec.report_url {
                Some(url_string) => Url::parse(url_string).ok(),
                None => None,
            }
        },
        None => None,
    };

    Ok(config)
}


#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use super::{Config, CONFIG_FILE_NAME, MonitorSpec};

    #[test]
    fn config_monitor_connection() {
        let config: Config = toml::from_str("monitor = \"Connection\"\n").unwrap();
        assert_eq!(config.monitor_spec, Some(MonitorSpec::Connection));
    }

    #[test]
    fn config_monitor_all() {
        let config: Config = toml::from_str("monitor=\"All\"\n").unwrap();
        assert_eq!(config.monitor_spec, Some(MonitorSpec::All))
    }

    #[test]
    fn config_monitor_ssids() {
        let config: Config = toml::from_str("[monitor]\nSSIDs=['ABC', 'DEF']\n").unwrap();
        let ssid_list = MonitorSpec::SSIDs(vec!["ABC".to_owned(), "DEF".to_owned()]);
        assert_eq!(config.monitor_spec, Some(ssid_list))
    }

    #[test]
    fn bundled_spec() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_dir = manifest_dir.parent().ok_or("Could not get parent dir")
            .expect("Could not get parent dir");
        let config_string = std::fs::read_to_string(root_dir.join(CONFIG_FILE_NAME))
            .unwrap();
        let config: Config = toml::from_str(&config_string).unwrap();
        assert_eq!(config.monitor_spec, Some(MonitorSpec::Connection));
        assert_eq!(config.report_spec.unwrap().period_seconds, Some(60));
    }

    #[test]
    fn config_with_report_spec() {
        let config: Config = toml::from_str("[report]\nperiod_seconds = 1\n").unwrap();
        assert_eq!(config.report_spec.unwrap().period_seconds, Some(1));
    }
}