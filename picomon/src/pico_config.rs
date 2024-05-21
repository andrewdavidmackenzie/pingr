#[allow(dead_code)]
pub(crate) enum MonitorSpec {
    /// Report status of all SSIDs that are detected at each monitoring moment
    All,
    /// Only report on the status of the connection (wifi or ethernet) used to send results
    Connection,
    /// Monitor this one SSID, with the supplied name and password
    Ssid(&'static str, &'static str),
}

#[allow(dead_code)]
pub(crate) struct ReportSpec {
    pub period_seconds: u64,
    pub base_url: &'static str,
}

#[allow(dead_code)]
pub struct Config {
    pub monitor: MonitorSpec,
    pub report: ReportSpec,
}
