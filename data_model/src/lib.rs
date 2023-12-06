use std::fmt::{Display, Formatter};

// put under option
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize)]
pub struct Stats {
    power_percent: u8
}

#[derive(Serialize, Deserialize)]
pub struct ConnectionReport {
    ssid: String,
    stats: Option<Stats>,
}

#[derive(Serialize, Deserialize)]
pub struct MonitorReport {
    pub device_id: DeviceId,
    pub local_time: Option<String>,
    pub connections: Vec<ConnectionReport>,
}

impl Display for MonitorReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tDeviceId = {}", self.device_id)
    }
}