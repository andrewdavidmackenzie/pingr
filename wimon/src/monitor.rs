use config::Config;
#[cfg(feature = "ssids")]
use config::MonitorSpec;
use curl::easy::Easy;
use data_model::{Connection, DeviceId, MonitorReport, ReportType};
#[cfg(feature = "ssids")]
use data_model::{ConnectionReport, Stats};
use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use serde_json::json;
use std::io;
use std::io::Read;
use std::process::Command;
use std::sync::mpsc::Receiver;
#[cfg(feature = "ssids")]
use wifiscanner::Wifi;

pub(crate) fn monitor_loop(config: Config, term_receiver: Receiver<()>) -> Result<(), io::Error> {
    let device_id = get_device_id()?;
    println!("Device ID = {device_id}");

    // A "sleep", interruptible by receiving a message to exit. Normal looping will produce
    // a timeout error, in which case send the periodic report.
    while term_receiver.recv_timeout(config.period_duration).is_err() {
        // Avoid failing on one error
        let _ = send_report(&config, &device_id, ReportType::OnGoing, &measure(&config)?);
    }

    // Tell the server that this device is stopping sending of reports
    send_report(&config, &device_id, ReportType::Stop, &measure(&config)?)
}

#[cfg(feature = "ssids")]
fn add_report(report: &mut MonitorReport, wifi: &Wifi) {
    report.connections.push(ConnectionReport {
        connection: Connection::SSID(wifi.ssid.clone()),
        stats: Some(Stats {
            power_dbs: wifi.signal_level.parse::<i16>().unwrap_or(0),
        }),
    });
}
#[cfg_attr(not(feature = "ssids"), allow(unused_variables))]
fn measure(config: &Config) -> Result<MonitorReport, io::Error> {
    let ssid = get_ssid().map_err(|e| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not get SSID: '{e}'"),
        )
    })?;

    #[cfg_attr(not(feature = "ssids"), allow(unused_mut))]
    let mut report = MonitorReport {
        connection_used: Connection::SSID(ssid.clone()),
        connections: vec![],
    };

    #[cfg(feature = "ssids")]
    match &config.monitor.as_ref().unwrap_or(&MonitorSpec::Connection) {
        MonitorSpec::All => {
            let wifis = wifiscanner::scan().unwrap_or_default();
            for wifi in wifis {
                add_report(&mut report, &wifi);
            }
        }
        MonitorSpec::Connection => {
            let wifis = wifiscanner::scan().unwrap_or_default();
            for wifi in wifis {
                if wifi.ssid == ssid {
                    add_report(&mut report, &wifi);
                }
            }
        }
    };

    Ok(report)
}

fn send_report(
    config: &Config,
    device_id: &DeviceId,
    report_type: ReportType,
    report: &MonitorReport,
) -> Result<(), io::Error> {
    let report_url = config.report_url.as_ref().map(|p| {
        p.join(&format!(
            "report/{}?device_id={}&connection={}&period={}",
            report_type.to_string().to_ascii_lowercase(),
            device_id,
            report.connection_used,
            config.period_duration.as_secs()
        ))
        .unwrap()
    });

    let mut data = Vec::new();
    if let Some(url) = &report_url {
        let json_string = format!("report={}", json!(report));
        let mut post_data = json_string.as_bytes();
        let mut easy = Easy::new();
        let result;
        easy.url(url.as_str()).map_err(|_| {
            io::Error::new(io::ErrorKind::NotFound, "Could not set url on curl request")
        })?;
        {
            easy.post(true).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Could not set POST on curl request",
                )
            })?;
            easy.post_fields_copy(post_data).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Could not add POST data on curl request",
                )
            })?;
            easy.post_field_size(post_data.len() as u64).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Could not set POST field size on curl request",
                )
            })?;
            let mut transfer = easy.transfer();
            transfer
                .read_function(|buf| Ok(post_data.read(buf).unwrap()))
                .map_err(|_| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        "Could not read data for curl request",
                    )
                })?;
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
            }
            Err(_) => eprintln!("Error reporting to '{}': skipping report", url.as_str()),
        }
        result
            .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not perform curl request"))
    } else {
        println!("Local Status: \n{report}");
        Ok(())
    }
}

fn get_device_id() -> Result<DeviceId, io::Error> {
    let mut builder = IdBuilder::new(Encryption::SHA256);
    builder
        .add_component(HWIDComponent::CPUID)
        .add_component(HWIDComponent::SystemID);
    builder
        .build("device_id")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not build unique device_id"))
}

#[cfg(target_os = "macos")]
fn get_ssid() -> Result<String, io::Error> {
    let output = Command::new("/usr/sbin/networksetup")
        .arg("-getairportnetwork")
        .arg("en0")
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let data = String::from_utf8_lossy(&output.stdout);

    parse_ssid(&data)
}

#[cfg(target_os = "macos")]
fn parse_ssid(data: &str) -> Result<String, io::Error> {
    for line in data.lines() {
        let mut pair = line.trim().split(':');
        if pair.next().unwrap() == "Current Wi-Fi Network" {
            return Ok(pair.next().unwrap().trim().to_owned());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Could not parse SSID name: '{data}'"),
    ))
}

// This will need improving for the case when there are multiple interfaces
#[cfg(target_os = "linux")]
fn get_ssid() -> Result<String, io::Error> {
    let output = Command::new("iw")
        .arg("dev")
        .output()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not execute 'iw'"))?;
    let data = String::from_utf8_lossy(&output.stdout);
    parse_ssid(&data)
}

#[cfg(target_os = "linux")]
fn parse_ssid(data: &str) -> Result<String, io::Error> {
    for line in data.lines() {
        let mut pair = line.trim().split(' ');
        if pair.next().unwrap() == "ssid" {
            return Ok(pair.next().unwrap().trim().to_owned());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not parse SSID name",
    ))
}
