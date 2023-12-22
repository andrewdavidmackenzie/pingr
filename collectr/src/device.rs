use worker::*;
use std::fmt::{Display, Formatter};
use worker::durable_object;
use data_model::{DeviceId, MonitorReport, ReportType};
use crate::device::DeviceState::{Offline, Reporting, NotReporting};
// use data_model::MonitorReport;

#[derive(Debug, PartialEq)]
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
        // Retrieve previous device_state. If not start in NotReporting
        // How to load previous stateif this is NOT a new DO?
//        let initial_state = state.storage().get("device_state").await.unwrap_or(NotReporting);
        Self {
            state,
            env,
//            device_state: initial_state,
            device_state: NotReporting,
        }
    }

    async fn fetch(&mut self, mut req: Request) -> Result<Response> {
        console_log!("DO request {} - Previous state: {}", req.path(), self.device_state);

        let form = req.form_data().await?;
        match form.get("report") {
            Some(report_entry) => match report_entry {
                FormEntry::Field(report_string) => {
                    let report_json: serde_json::Result<MonitorReport> = serde_json::from_str(&report_string);
                    match report_json {
                        Ok(report) => {
                            self.process_report(report, false);
                            Response::ok("Report Processed")
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

    async fn alarm(&mut self) -> Result<Response> {
        Response::ok("ok")
    }
}

impl Device {
    fn new_state(&mut self, id: &DeviceId, new_state: DeviceState) {
        self.device_state = new_state;
        console_log!("Device ID: {} - Now in {} state", id, self.device_state);
    }

    fn process_report(&mut self, report: MonitorReport, alarm: bool) {
        match (report.report_type, alarm) {
            (_, true) => {  // report overdue
                match &self.device_state {
                    NotReporting => console_warn!("Report overdue with device in NotReporting state"),
                    Reporting => {
                        console_log!("Device ID: {} - Report overdue", report.device_id);
                        self.new_state(&report.device_id, Offline);
                    },
                    Offline => console_warn!("Report overdue with device in Offline state"),
                }
            },
            (ReportType::Start, _) => { // Start report
                if self.device_state == Reporting {
                    console_warn!("Start Report with device in Reporting state");
                }
                self.new_state(&report.device_id, Reporting);
            },
            (ReportType::Stop, _) => { // Stop report
                if self.device_state == NotReporting {
                    console_warn!("Stop Report with device in NotReporting state");
                }
                self.new_state(&report.device_id, NotReporting);
            }
            (ReportType::OnGoing, _) => { // OnGoing report
                if self.device_state == NotReporting {
                    console_warn!("OnGoing Report with device in NotReporting state");
                }
                self.new_state(&report.device_id, Reporting);
            }
        }
    }
}
