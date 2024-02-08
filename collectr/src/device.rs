use crate::device::DeviceState::{New, Offline, Reporting, Stopped};
use data_model::MonitorReport;
use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use worker::durable_object;
use worker::*;

const MARGIN_SECONDS: u64 = 5;

pub const STATE_CHANGES_QUEUE: &str = "STATE_CHANGES";

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
            New => write!(f, "New"),
            Stopped => write!(f, "Stopped"),
            Reporting => write!(f, "Reporting"),
            Offline => write!(f, "Offline"),
        }
    }
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct StateChange {
    pub id: String,
    pub new_state: DeviceState,
    pub connection: Option<String>,
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

        self.load_state().await;

        let mut period = None;
        let mut connection = None;
        let url = req.url().unwrap();
        for query_pair in url.query_pairs() {
            match query_pair.0 {
                Cow::Borrowed("connection") => connection = Some(query_pair.1),
                Cow::Borrowed("period") => period = query_pair.1.parse::<u64>().ok(),
                _ => {}
            }
        }

        match req.method() {
            Method::Post => {
                let form = req.form_data().await?;
                match form.get("report") {
                    Some(report_entry) => match report_entry {
                        FormEntry::Field(report_string) => {
                            let report_json: serde_json::Result<MonitorReport> =
                                serde_json::from_str(&report_string);
                            match report_json {
                                Ok(report) => {
                                    self.process_report(
                                        report_type,
                                        period,
                                        Some(report),
                                        connection,
                                    )
                                    .await
                                }
                                Err(_) => Response::error("Could not deserialize report", 400),
                            }
                        }
                        _ => Response::error("Unexpected File attached to report", 400),
                    },
                    _ => Response::error("Unexpected FormEntry in report FormData", 400),
                }
            }
            Method::Get => {
                self.process_report(report_type, period, None, connection)
                    .await
            }
            _ => Response::error("Unexpected HTTP Method used", 400),
        }
    }

    // A DO alarm expired - which should indicate that the device didn't report in time
    async fn alarm(&mut self) -> Result<Response> {
        console_log!("\nDO ID: {}", self.state.id().to_string());
        self.load_state().await;
        self.process_report("alarm", None, None, None).await
    }
}

impl Device {
    // Process a new report or an alarm - implementing the state machine, changing to the new state when required
    // and logging console warnings for states that should not happen if everything is working perfectly
    async fn process_report(
        &mut self,
        report_type: &str,
        period_seconds: Option<u64>,
        _report: Option<MonitorReport>,
        connection: Option<Cow<'_, str>>,
    ) -> Result<Response> {
        console_log!("Event: {}", report_type);

        // Note: `New` is not one of the possible states set below, so if this is the first time the DO for this
        // device runs it MUST result in a different state (Reporting would be normal, but others in error cases)
        // and so the new state (`New` not being one of them) MUST be stored and a state change event generated
        match report_type {
            "ongoing" => {
                // An OnGoing report was received
                if let Some(period) = period_seconds {
                    self.state
                        .storage()
                        .set_alarm(((period + MARGIN_SECONDS) * 1000) as i64)
                        .await?;
                }
                self.new_state(Reporting, connection).await?;
            }
            "stop" => {
                // A Stop report was received
                if self.device_state == Stopped {
                    console_warn!("Stop Report with device in Stopped state");
                }
                self.state.storage().delete_alarm().await?;
                self.new_state(Stopped, connection).await?;
            }
            _ => match &self.device_state {
                // alarm was sent - so an expected report didn't arrive by the expected time
                New => console_warn!("Report overdue with device in New state"),
                Stopped => console_warn!("Report overdue with device in Stopped state"),
                Offline => console_warn!("Report overdue with device in Offline state"),
                Reporting => self.new_state(Offline, connection).await?,
            },
        }

        Response::ok(format!(
            "Device ID: {} State: {}",
            self.state.id().to_string(),
            self.device_state
        ))
    }

    // change the state of the tracked device to the new state, if it is different from the current state
    // then store the state for use in future instances of this DurableObject
    async fn new_state(
        &mut self,
        new_state: DeviceState,
        connection: Option<Cow<'_, str>>,
    ) -> Result<()> {
        if self.device_state != new_state {
            let id = &self.state.id().to_string();

            console_log!(
                "Device DO: State transition from {} to {}",
                self.device_state,
                new_state
            );

            self.device_state = new_state;

            // Store the state in the DO's storage for next time around
            self.state
                .storage()
                .put("device_state", &self.device_state)
                .await?;

            // Send the new state to the STATE_CHANGES queue for background processing
            let queue = self.env.queue(STATE_CHANGES_QUEUE)?;
            let state_change = StateChange {
                id: id.to_string(),
                new_state: self.device_state.clone(),
                connection: connection.map(|s| s.to_string()),
            };
            queue.send(&state_change).await?;
        }

        Ok(())
    }

    // Retrieve previous device_state from DO storage. If not present (first time!), then [New]
    async fn load_state(&mut self) {
        self.device_state = self
            .state
            .storage()
            .get("device_state")
            .await
            .unwrap_or(New);
        console_log!("State: {}", self.device_state);
    }
}
