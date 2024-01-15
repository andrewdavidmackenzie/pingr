use std::fmt::{Display, Formatter};

// put under option
use serde_derive::{Serialize, Deserialize};

pub enum DeviceId {
    MAC([u8;6])
}

impl Display for DeviceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceId::MAC(mac) => write!(f, "MAC({:?})", mac)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stats {
    pub power_dbs: i16
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Connection {
    SSID(String),
    Ethernet
}

impl Display for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Connection::Ethernet => write!(f, "\tEthernet"),
            Connection::SSID(ssid) => write!(f, "\tSSID = {}", ssid),
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
    OnGoing
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
            connection_used: Connection::Ethernet,
            connections: vec![],
        }
    }
}

impl Display for MonitorReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tConnection Used = {}", self.connection_used)
    }
}