use worker::*;
use std::fmt::{Display, Formatter};
use serde_derive::{Deserialize, Serialize};
use worker::durable_object;
use data_model::{DeviceId, MonitorReport, ReportType};
use crate::device::DeviceState::{Offline, Reporting, NotReporting};
// use data_model::MonitorReport;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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
        console_log!("DO - Creating new instance");
        Self {
            state,
            env,
            device_state: NotReporting,
        }
    }

    async fn fetch(&mut self, mut req: Request) -> Result<Response> {
        // Retrieve previous device_state. If not start in NotReporting
        self.device_state = self.state.storage().get("device_state").await.unwrap_or(NotReporting);

        console_log!("DO request {} - Previous state: {}", req.path(), self.device_state);

        let form = req.form_data().await?;
        match form.get("report") {
            Some(report_entry) => match report_entry {
                FormEntry::Field(report_string) => {
                    let report_json: serde_json::Result<MonitorReport> = serde_json::from_str(&report_string);
                    match report_json {
                        Ok(report) => self.process_report(report, false).await,
                        Err(_) => Response::error("Could not deserialize report", 400)
                    }
                },
                _ => Response::error("Unexpected File attached to report", 400)
            }
            _ => Response::error("Unexpected FormEntry in report FormData", 400)
        }
    }

    async fn alarm(&mut self) -> Result<Response> {
        Response::ok("ok")
    }
}

impl Device {
    // change the state of the tracked device to the new state, if it is different from the current state
    // then store the state for use in future instances of this DurableObject
    async fn new_state(&mut self, id: &DeviceId, new_state: DeviceState) -> Result<()> {
        if self.device_state != new_state {
            console_log!("Device ID: {} - State transition from {} to {}", id,
                self.device_state, new_state);
            self.device_state = new_state;
            self.state.storage().put("device_state", &self.device_state).await
        } else {
            Ok(())
        }
    }

    // Process a new report or alarm - implementing the state machine, changing to the new state when required
    // and logging console warnings for states that should not happen if everything is working perfectly
    async fn process_report(&mut self, report: MonitorReport, alarm: bool) -> Result<Response> {
        match (report.report_type, alarm) {
            (_, true) => {  // report overdue
                match &self.device_state {
                    NotReporting => console_warn!("Report overdue with device in NotReporting state"),
                    Reporting => {
                        console_log!("Device ID: {} - Report overdue", report.device_id);
                        self.new_state(&report.device_id, Offline).await?;
                    },
                    Offline => console_warn!("Report overdue with device in Offline state"),
                }
            },
            (ReportType::Start, _) => { // Start report
                if self.device_state == Reporting {
                    console_warn!("Start Report with device in Reporting state");
                }
                self.new_state(&report.device_id, Reporting).await?;
            },
            (ReportType::Stop, _) => { // Stop report
                if self.device_state == NotReporting {
                    console_warn!("Stop Report with device in NotReporting state");
                }
                self.new_state(&report.device_id, NotReporting).await?;
            }
            (ReportType::OnGoing, _) => { // OnGoing report
                if self.device_state == NotReporting {
                    console_warn!("OnGoing Report with device in NotReporting state");
                }
                self.new_state(&report.device_id, Reporting).await?;
            }
        }

        Response::ok("Report Processed")
    }
}
