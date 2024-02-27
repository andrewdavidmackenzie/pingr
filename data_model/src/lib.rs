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

/// [Device] implements a Cloudflare DistributedObject that tracks the state of one monitoring device.
/// The state is maintained inside the DO itself, in case it is called multiple times without being
/// shutdown between them, but is also stored and loaded from DO storage.
///
/// It uses the `alarm` feature of DistributedObjects to put the devices state into `NotReporting` if
/// a report is overdue.
///
/// It sends any state change to the `STATE_CHANGES` queue, where a worker can do further processing
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum DeviceState {
    /// New signifies that the state for this Device has not been loaded from storage yet
    /// and it maybe the first time this DO for it runs, hence there is nothing in storage
    /// This ensures that the first time the DO runs, as different state MUST result and the
    /// initial (real) state is written to storage and event generated as the state changed
    New,
    /// The device stopped reporting, and is not considered offline
    Stopped,
    /// The device is reporting, and more reports should be expected, on-time
    Reporting,
    /// The device should be reporting, but a report didn't arrive on-time
    Offline,
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::Stopped => write!(f, "Stopped"),
            Self::Reporting => write!(f, "Reporting"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct StateChange {
    pub id: String,
    pub state: DeviceState,
    pub connection: Option<String>,
    pub timestamp: u64, // millis in Unix EPOCH
}
