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

#[derive(Serialize, Deserialize)]
pub struct Stats {
    power_percent: u8
}

#[derive(Serialize, Deserialize)]
pub struct ConnectionReport {
    ssid: String,
    stats: Option<Stats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ReportType {
    Start,
    Stop,
    OnGoing
}

impl Display for ReportType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportType::Start => write!(f, "Start"),
            ReportType::Stop => write!(f, "Stop"),
            ReportType::OnGoing => write!(f, "OnGoing"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MonitorReport {
    pub report_type: ReportType,
    pub period_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_time: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<ConnectionReport>,
}

impl Default for MonitorReport {
    fn default() -> Self {
        MonitorReport {
            report_type: ReportType::Start,
            period_seconds: 0,
            local_time: None,
            connections: vec![],
        }
    }
}
impl Display for MonitorReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tReportType = {}", self.report_type)
    }
}