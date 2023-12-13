use std::{env, io};
use std::path::PathBuf;
use std::time::Duration;
use mac_address::get_mac_address;
use url::Url;
use serde_json::json;
use std::io::Read;
use curl::easy::Easy;

// put under option
use serde_derive::{Serialize, Deserialize};

use data_model::{DeviceId, MonitorReport, ReportType};
use data_model::ReportType::OnGoing;
use ctrlc;
use std::sync::mpsc::{channel, Receiver};

const CONFIG_FILE_NAME: &str = "wimon.toml";

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ReportSpec {
    period_seconds: Option<u64>,
    #[serde(rename="url")]
    report_url: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
struct Config {
    #[serde(rename="monitor")]
    monitor_spec: Option<MonitorSpec>,
    #[serde(rename="report")]
    report_spec: Option<ReportSpec>,
    #[serde(skip)]
    period_duration: Duration,
    #[serde(skip)]
    report_url: Option<Url>,
}

fn main() -> Result<(), io::Error> {
    let config_file_path = find_config_file(CONFIG_FILE_NAME)?;
    let config = read_config(&config_file_path)?;
    println!("Config file loaded from: \"{}\"", config_file_path.display());
    println!("Monitor: {:?}", config.monitor_spec);

    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    monitor_loop(config, rx)?;

    Ok(())
}

fn monitor_loop(config: Config, term_receiver: Receiver<()>) -> Result<(), io::Error> {
    let device_id = get_device_id()?;
    let report_url = config.report_url.as_ref()
        .map(|p| p.join("report").unwrap());

    if let Some(url) = &report_url {
        start_reporting(url, &device_id)
    }

    while term_receiver.recv_timeout(config.period_duration).is_err() {
        let report = MonitorReport {
            report_type: OnGoing,
            device_id: device_id.clone(),
            period_seconds: config.period_duration.as_secs(),
            local_time: None,
            connections: vec![]
        };

        if let Some(url) = &report_url {
            match send_report(url, &report) {
                Ok(_) => println!("Sent {:?} report to: {url}", report.report_type),
                Err(_) => eprintln!("Error reporting to '{}': skipping report", url.as_str()),
            }
        } else {
            println!("Local Status: \n{report}");
        }
    }

    if let Some(url) = &report_url {
        stop_reporting(url, &device_id)
    }

    println!("Exiting");

    Ok(())
}

fn start_reporting(report_url: &Url, device_id: &DeviceId) {
    let report = MonitorReport {
        report_type: ReportType::Start,
        device_id: device_id.clone(),
        period_seconds: 0,
        local_time: None,
        connections: vec![]
    };

    match send_report(report_url, &report) {
        Ok(_) => println!("Sent {:?} report to: {report_url}", report.report_type),
        Err(_) => eprintln!("Error reporting to '{}': skipping report", report_url.as_str()),
    }
}

fn stop_reporting(report_url: &Url, device_id: &DeviceId) {
    let report = MonitorReport {
        report_type: ReportType::Stop,
        device_id: device_id.clone(),
        period_seconds: 0,
        local_time: None,
        connections: vec![]
    };

    match send_report(report_url, &report) {
        Ok(_) => println!("Sent {:?} report to: {report_url}", report.report_type),
        Err(_) => eprintln!("Error reporting to '{}': skipping report", report_url.as_str()),
    }
}

fn send_report(report_url: &Url, report: &MonitorReport) -> Result<(), curl::Error> {
    let json_string = format!("report={}", json!(report).to_string());
    let mut post_data = json_string.as_bytes();
    let mut easy = Easy::new();
    easy.url(report_url.as_str())?;
    easy.post(true)?;
    easy.post_fields_copy(post_data)?;
    easy.post_field_size(post_data.len() as u64)?;
    let mut transfer = easy.transfer();
    transfer.read_function(|buf| { Ok(post_data.read(buf).unwrap()) })?;
    transfer.perform()
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
    use crate::{Config, CONFIG_FILE_NAME, MonitorSpec};

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