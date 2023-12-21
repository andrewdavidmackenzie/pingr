use worker::*;
use std::fmt::{Display, Formatter};
use worker::durable_object;
use data_model::MonitorReport;
// use data_model::MonitorReport;

#[derive(Debug)]
enum DeviceState {
    /// The device stopped reporting, and is not considered offline
    NotReporting,
    /// The device is reporting, and more reports should be expected, on-time
    Reporting,
    /// The device should be reporting, but a report didn't arrive on-time
    Offline,
}

impl Display for DeviceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::NotReporting => write!(f, "NotReporting"),
            DeviceState::Reporting => write!(f, "Reporting"),
            DeviceState::Offline => write!(f, "Offline"),
        }
    }
}

/// TODO state machine docs for [Device] here

#[durable_object]
pub struct Device {
    /// state specific to the Durable Object, including methods for accessing storage.
    state: State,
    /// contains any bindings you have associated with the Worker when you uploaded it
    env: Env,
    /// The [DeviceState] this Device is currently in
    device_state: DeviceState,
}

#[durable_object]
impl DurableObject for Device {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            device_state: DeviceState::NotReporting,
        }
    }

    async fn fetch(&mut self, mut req: Request) -> Result<Response> {
        console_log!("DO got request {} - Previous state: {}", req.path(), self.device_state);

        let form = req.form_data().await?;
        match form.get("report") {
            Some(report_entry) => match report_entry {
                FormEntry::Field(report_string) => {
                    let report_json: serde_json::Result<MonitorReport> = serde_json::from_str(&report_string);
                    match report_json {
                        Ok(report) => {
                            console_debug!("Next report expected in: {}s", report.period_seconds);
                            Response::ok(&format!("Report from: {}", report.device_id.to_string()))
                        },
                        Err(e) => {
                            console_error!("Could not deserialize report: {e}");
                            console_error!("{}", report_string);
                            Response::error("Could not deserialize report", 400)
                        }
                    }
                },
                _ => {
                    console_error!("Unexpected File attached to report");
                    Response::error("Unexpected File", 400)
                }
            }
            _ => {
                console_error!("Unexpected FormEntry");
                Response::error("Unexpected FormEntry", 400)
            }
        }
    }
}
