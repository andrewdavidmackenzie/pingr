use std::fmt::{Display, Formatter};

// put under option
use serde_derive::{Deserialize, Serialize};

pub type DeviceId = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    pub power_dbs: i16,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Connection {
    SSID(String),
    Ethernet(String),
}

impl Display for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Connection::Ethernet(mac) => write!(f, "\tethernet={mac}"),
            Connection::SSID(ssid) => write!(f, "\tssid={ssid}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionReport {
    pub connection: Connection,
    pub stats: Option<Stats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ReportType {
    Stop,
    OnGoing,
}

impl Display for ReportType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportType::Stop => write!(f, "Stop"),
            ReportType::OnGoing => write!(f, "OnGoing"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MonitorReport {
    pub connection_used: Connection,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<ConnectionReport>,
}

impl Default for MonitorReport {
    fn default() -> Self {
        MonitorReport {
            connection_used: Connection::Ethernet("default".to_string()),
            connections: vec![],
        }
    }
}

impl Display for MonitorReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tConnection Used = {}", self.connection_used)
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct DeviceDetails {
    pub friendly_name: Option<String>,
}
