use worker::*;
use data_model::MonitorReport;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
    // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
    // routes to access and share using the `ctx.data()` method.
    let router = Router::new();

    router
        .post_async("/report", |mut req, _ctx| async move {
            let form = req.form_data().await?;
            match form.get("report") {
                Some(report_entry) => match report_entry {
                    FormEntry::Field(report_string) => {
                        let report_json : serde_json::Result<MonitorReport> = serde_json::from_str(&report_string);
                        match report_json {
                            Ok(report) => {
                                console_debug!("Received {:?} report from Device: {}",
                                    report.report_type, report.device_id);
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
