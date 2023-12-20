use worker::*;
use data_model::MonitorReport;

#[durable_object]
pub struct Device {
    state: State,
    env: Env, // access `Env` across requests, use inside `fetch`
}

#[durable_object]
impl DurableObject for Device {
    fn new(state: State, env: Env) -> Self {
        Self {
            state,
            env,
        }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // do some work when a worker makes a request to this DO
        Response::ok("Hello Device")
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
    // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
    // routes to access and share using the `ctx.data()` method.
    let router = Router::new();

    router
        .post_async("/report", |mut req, ctx| async move {
            let form = req.form_data().await?;
            match form.get("report") {
                Some(report_entry) => match report_entry {
                    FormEntry::Field(report_string) => {
                        let report_json : serde_json::Result<MonitorReport> = serde_json::from_str(&report_string);
                        match report_json {
                            Ok(report) => {
                                let namespace = ctx.durable_object("DEVICES")?;
                                let _stub = namespace.id_from_name(&report.device_id.to_string())?.get_stub()?;
                                console_log!("Received {:?} report from Device: {}. Next report expected in {} seconds",
                                    report.report_type, report.device_id, report.period_seconds);
                                Response::ok("OK - Report Accepted")
                            }
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
        })
        .run(req, env).await
}
