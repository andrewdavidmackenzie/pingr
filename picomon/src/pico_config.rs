#[allow(dead_code)]
pub(crate) enum MonitorSpec {
    /// Report status of all SSIDs that are detected at each monitoring moment
    All,
    /// Only report on the status of the connection (wifi or ethernet) used to send results
    Connection,
    /// Monitor this one SSID, with the supplied name and password
    SSID(&'static str, &'static str),
}

#[allow(dead_code)]
pub(crate) struct ReportSpec {
    pub period_seconds: Option<u64>,
    pub base_url: Option<&'static str>,
}

#[allow(dead_code)]
pub(crate) struct Config {
    pub monitor: Option<MonitorSpec>,
    pub report: Option<ReportSpec>,
}
