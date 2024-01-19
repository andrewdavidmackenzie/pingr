use worker::*;
use std::fmt::{Display, Formatter};
use serde_derive::{Deserialize, Serialize};
use worker::durable_object;
use data_model::MonitorReport;
use crate::device::DeviceState::{New, Offline, Reporting, Stopped};
use std::borrow::Cow;

const MARGIN_SECONDS: u64 = 5;

const DEVICE_STATUS_KV_NAMESPACE: &str = "DEVICE_STATUS";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
enum DeviceState {
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
            DeviceState::New => write!(f, "New"),
            DeviceState::Stopped => write!(f, "Stopped"),
            DeviceState::Reporting => write!(f, "Reporting"),
            DeviceState::Offline => write!(f, "Offline"),
        }
    }
}

#[durable_object]
#[allow(dead_code)]
pub struct Device {
    /// state specific to the Durable Object, including methods for accessing storage.
    state: State,
    /// contains any bindings you have associated with the Worker when you uploaded it
    env: Env, // TODO -determine if needed or not and remove if not
    /// The [DeviceState] this Device is currently in
    device_state: DeviceState,
}

#[durable_object]
impl DurableObject for Device {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
            device_state: New,
        }
    }

    async fn fetch(&mut self, mut req: Request) -> Result<Response> {
        console_log!("\nDO ID: {}", self.state.id().to_string());

        let path = req.path();
        let report_type = path.split('/').nth(2).unwrap();

        // Retrieve previous device_state. If not present (first time!), then New
        self.device_state = self.state.storage().get("device_state").await.unwrap_or(New);
        console_log!("State: {}", self.device_state);

        let mut period = None;
        let url = req.url().unwrap();
        for query_pair in url.query_pairs() {
            if let Cow::Borrowed("period") = query_pair.0 {
                period = query_pair.1.parse::<u64>().ok();
            }
        }

        match req.method() {
            Method::Post => {
                let form = req.form_data().await?;
                match form.get("report") {
                    Some(report_entry) => match report_entry {
                        FormEntry::Field(report_string) => {
                            let report_json: serde_json::Result<MonitorReport> = serde_json::from_str(&report_string);
                            match report_json {
                                Ok(report) => self.process_report(report_type, period, Some(report)).await,
                                Err(_) => Response::error("Could not deserialize report", 400)
                            }
                        },
                        _ => Response::error("Unexpected File attached to report", 400)
                    }
                    _ => Response::error("Unexpected FormEntry in report FormData", 400)
                }
            },
            Method::Get => self.process_report(report_type, period, None).await,
            _ => Response::error("Unexpected HTTP Method used", 400)
        }
    }

    // A DO alarm expired - which should indicate that the device didn't report in time
    async fn alarm(&mut self) -> Result<Response> {
        console_log!("\nDO ID: {}", self.state.id().to_string());

        // Retrieve previous device_state. If not present (first time!), then start in New
        self.device_state = self.state.storage().get("device_state").await.unwrap_or(New);
        console_log!("State: {}", self.device_state);

        self.process_report("alarm", None, None).await
    }
}

impl Device {
    // Process a new report or an alarm - implementing the state machine, changing to the new state when required
    // and logging console warnings for states that should not happen if everything is working perfectly
    async fn process_report(&mut self, report_type: &str, period_seconds: Option<u64>, _report: Option<MonitorReport>)
        -> Result<Response> {
        console_log!("Event: {}", report_type);

        // Note: `New` is not one of the possible states set below, so if this is the first time the DO for this
        // device runs it MUST result in a different state (Reporting would be normal, but others in error cases)
        // and so the new state (`New` not being one of them) MUST be stored and a state change event generated
        match report_type {
            "ongoing" => { // OnGoing report
                if let Some(period) = period_seconds {
                    self.state.storage().set_alarm(((period + MARGIN_SECONDS) * 1000) as i64).await?;
                }
                self.new_device_state(Reporting).await?;
            },
            "stop" => { // Stop report
                if self.device_state == Stopped {
                    console_warn!("Stop Report with device in Stopped state");
                }
                self.state.storage().delete_alarm().await?;
                self.new_device_state(Stopped).await?;
            }
            _ => {
                match &self.device_state {
                    New => console_warn!("Report overdue with device in New state"),
                    Stopped => console_warn!("Report overdue with device in Stopped state"),
                    Offline => console_warn!("Report overdue with device in Offline state"),
                    Reporting => self.new_device_state(Offline).await?,
                }
            }
        }

        Response::ok(format!("Device ID: {} State: {}", self.state.id().to_string(), self.device_state))
    }

    // change the state of the tracked device to the new state, if it is different from the current state
    // then store the state for use in future instances of this DurableObject
    async fn new_device_state(&mut self, new_state: DeviceState) -> Result<()> {
        if self.device_state != new_state {
            console_log!("State transition from {} to {}", self.device_state, new_state);
            self.device_state = new_state;
            // Store the state in the DO's storage for next time around
            self.state.storage().put("device_state", &self.device_state).await?;
            // Store the state in KV store that can be read elsewhere
            let kv = self.env.kv(DEVICE_STATUS_KV_NAMESPACE)?;
            kv.put(&self.state.id().to_string(), &self.device_state)?.execute().await?;
            Ok(())
        } else {
            Ok(())
        }
    }
}
