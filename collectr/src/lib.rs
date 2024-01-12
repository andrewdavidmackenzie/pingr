use worker::*;

mod device;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/report/:type/:id", |req, ctx| async move {
            let headers = req.headers();
            if let Ok(Some(ip)) = headers.get("CF-Connecting-IP") {
                console_log!("Source IP = {:?}", ip);
            }
            match ctx.param("id") {
                Some(device_id) => {
                    let namespace = ctx.durable_object("DEVICES")?;
                    let stub = namespace.id_from_name(&device_id)?.get_stub()?;
                    stub.fetch_with_request(req).await
                }
                _ => Response::error("Bad Request - missing device id", 400),
            }
        })
        .run(req, env)
        .await
}
