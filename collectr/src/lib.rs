use worker::*;
use data_model::MonitorReport;

mod device;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Create an instance of the Router, which can use parameters (/user/:name) or wildcard values
    // (/file/*pathname). Alternatively, use `Router::with_data(D)` and pass in arbitrary data for
    // routes to access and share using the `ctx.data()` method.
    let router = Router::new();

    router
        .post_async("/report/:id", |mut req, ctx| async move {
            match ctx.param("id") {
                Some(device_id) => {
                    let namespace = ctx.durable_object("DEVICES")?;
                    let stub = namespace.id_from_name(&device_id)?.get_stub()?;
                    console_debug!("Received report from Device: {}. Passing to DO", device_id);
                    stub.fetch_with_str(&format!("http://dummy.com/report/{}", device_id)).await
                }
                _ => Response::error("Bad Request - missing device id", 400)
            }
        })
        .run(req, env).await
}
