use std::{env, io};
use std::path::PathBuf;
use std::time::Duration;
use mac_address::get_mac_address;
use url::Url;
use serde_json::json;
use std::io::Read;
use curl::easy::Easy;
use std::process::Command;

// put under option
use serde_derive::{Serialize, Deserialize};

use data_model::{Connection, DeviceId, MonitorReport, ReportType, ConnectionReport, Stats};
use ctrlc;
use std::sync::mpsc::{channel, Receiver};

const CONFIG_FILE_NAME: &str = "wimon.toml";

#[cfg(feature = "ssids")]
use wifiscanner;

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
    #[serde(rename="base_url")]
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
    println!("Monitor: {:?}", config.monitor_spec.as_ref().unwrap_or(&MonitorSpec::Connection));

    let (tx, rx) = channel();
    ctrlc::set_handler(move || {
        println!("Control-C captured, sending Stop report");
        tx.send(()).expect("Could not send signal on channel.")
    })
        .expect("Error setting Ctrl-C handler");

    monitor_loop(config, rx)?;

    println!("Exiting");

    Ok(())
}

fn monitor_loop(config: Config, term_receiver: Receiver<()>) -> Result<(), io::Error> {
    let device_id = get_device_id()?;

    // Send initial report
    send_report(&config, &device_id, ReportType::OnGoing, &measure(&config)?)?;

    // A "sleep", interruptible by receiving a message to exit. Normal looping will produce
    // a timeout error, in which case send the periodic report.
    while term_receiver.recv_timeout(config.period_duration).is_err() {
        send_report(&config, &device_id, ReportType::OnGoing, &measure(&config)?)?;
    }

    // Tell the server that this device is stopping sending of reports
    send_report(&config, &device_id, ReportType::Stop, &measure(&config)?)?;

    Ok(())
}

fn measure(config: &Config) -> Result<MonitorReport, io::Error> {
    let ssid = get_ssid().map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not get SSID"))?;

    let mut report = MonitorReport {
        connection_used: Connection::SSID(ssid.clone()),
        connections: vec![]
    };

    #[cfg(feature = "ssids")]
    match &config.monitor_spec.as_ref().unwrap_or(&MonitorSpec::Connection) {
        MonitorSpec::All => {
            let wifis = wifiscanner::scan().unwrap_or(vec!());
            for wifi in wifis {
                report.connections.push(
                    ConnectionReport {
                        connection: Connection::SSID(wifi.ssid),
                        stats: Some( Stats {
                            power_dbs: wifi.signal_level.parse::<i16>().unwrap_or(0)
                        })
                });
            }
        },
        MonitorSpec::SSIDs(report_ssids) => {
            let wifis = wifiscanner::scan().unwrap_or(vec!());
            for wifi in wifis {
                if report_ssids.contains(&wifi.ssid) {
                    report.connections.push(
                        ConnectionReport {
                            connection: Connection::SSID(wifi.ssid),
                            stats: Some( Stats {
                                power_dbs: wifi.signal_level.parse::<i16>().unwrap_or(0)
                            })
                    });
                }
            }
        },
        MonitorSpec::Connection => {
            let wifis = wifiscanner::scan().unwrap_or(vec!());
            for wifi in wifis {
                if wifi.ssid == ssid {
                    report.connections.push(
                        ConnectionReport {
                            connection: Connection::SSID(wifi.ssid),
                            stats: Some( Stats {
                                power_dbs: wifi.signal_level.parse::<i16>().unwrap_or(0)
                            })
                        });
                }
            }
        }
    };

    Ok(report)
}

fn send_report(config: &Config, device_id: &DeviceId, report_type: ReportType, report: &MonitorReport)
    -> Result<(), io::Error> {
    let report_url = config.report_url.as_ref()
        .map(|p| p.join(&format!("report/{}?device_id={}&ssid={}&period={}", report_type.to_string().to_ascii_lowercase(),
                                 device_id.to_string(), report.connection_used, config.period_duration.as_secs())).unwrap());

    let mut data = Vec::new();
    if let Some(url) = &report_url {
        let json_string = format!("report={}", json!(report).to_string());
        let mut post_data = json_string.as_bytes();
        let mut easy = Easy::new();
        let result;
        easy.url(url.as_str()).map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not set url on curl request"))?;
        {
            easy.post(true).map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not set POST on curl request"))?;
            easy.post_fields_copy(post_data).map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not add POST data on curl request"))?;
            easy.post_field_size(post_data.len() as u64).map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not set POST field size on curl request"))?;
            let mut transfer = easy.transfer();
            transfer.read_function(|buf| { Ok(post_data.read(buf).unwrap()) })
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not read data for curl request"))?;
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })?;
            result = transfer.perform();
        }
        match result {
            Ok(_) => {
                println!("Sent {} report to: {}", report_type, url.host().unwrap());
                println!("Response: {}", String::from_utf8_lossy(&data));
            },
            Err(_) => eprintln!("Error reporting to '{}': skipping report", url.as_str()),
        }
        result.map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not perform curl request"))
    } else {
        println!("Local Status: \n{report}");
        Ok(())
    }
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

#[cfg(target_os = "macos")]
fn get_ssid() -> Result<String, io::Error> {
    let output = Command::new("/System/Library/PrivateFrameworks/Apple80211.\
         framework/Versions/Current/Resources/airport")
        .arg("-I")
        .output()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Command not found"))?;

    let data = String::from_utf8_lossy(&output.stdout);

    parse_ssid(&data)
}

#[cfg(target_os = "macos")]
fn parse_ssid(data: &str) -> Result<String, io::Error> {
    for line in data.lines() {
        let mut pair = line.trim().split(":");
        if pair.nth(0).unwrap() == "SSID" {
           return Ok(pair.nth(0).unwrap().trim().to_owned())
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Could not parse SSID name"))
}

// This will need improving for the case when there are multiple interfaces
#[cfg(target_os = "linux")]
fn get_ssid() -> Result<String, io::Error> {
    let output = Command::new("iw").arg("dev").output()
    .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not execute 'iw'"))?;
    let data = String::from_utf8_lossy(&output.stdout);
    parse_ssid(&data)
}

#[cfg(target_os = "linux")]
fn parse_ssid(data: &str) -> Result<String, io::Error> {
    for line in data.lines() {
        let mut pair = line.trim().split(" ");
        if pair.nth(0).unwrap() == "ssid" {
           return Ok(pair.nth(0).unwrap().trim().to_owned())
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Could not parse SSID name"))
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