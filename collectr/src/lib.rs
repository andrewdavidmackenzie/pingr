use worker::*;

mod device;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/report/:id", |mut req, ctx| async move {
            match ctx.param("id") {
                Some(device_id) => {
                    let namespace = ctx.durable_object("DEVICES")?;
                    let stub = namespace.id_from_name(&device_id)?.get_stub()?;
                    console_debug!(
                        "Received report from device_id: {}. Passing to DO",
                        device_id
                    );
                    stub.fetch_with_request(req.clone().unwrap()).await
                }
                _ => Response::error("Bad Request - missing device id", 400),
            }
        })
        .run(req, env)
        .await
}
