#[allow(dead_code)]
pub(crate) enum MonitorSpec {
    /// Report status of all SSIDs that are detected at each monitoring moment
    All,
    /// Only report on the status of the connection (wifi or ethernet) used to send results
    Connection,
    /// Monitor a specific list of supplied SSIDs by name
    //SSIDs(Vec<String>),
    /// Monitor this one SSID, with the supplied name and password
    SSID(&'static str, &'static str),
}

pub(crate) struct ReportSpec {
    pub period_seconds: Option<u64>,
    pub base_url: Option<&'static str>,
}

pub(crate) struct Config {
    pub monitor: Option<MonitorSpec>,
    pub report: Option<ReportSpec>,
}
