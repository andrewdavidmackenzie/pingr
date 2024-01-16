use worker::*;
use std::borrow::Cow;

mod device;

const DEVICE_ID_CONNECTION_MAPPING_KV_NAMESPACE: &str = "DEVICE_ID_CONNECTION_MAPPING";

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/report/:type", |req, ctx| async move {
            /*
            let headers = req.headers();
            if let Ok(Some(ip)) = headers.get("CF-Connecting-IP") {
                console_log!("Source IP = {:?}", ip);
            }
             */

            let mut device_id = None;
            let mut connection = None;
            let url = req.url().unwrap();
            for query_pair in url.query_pairs() {
                match query_pair.0 {
                    Cow::Borrowed("device_id") => device_id = Some(query_pair.1),
                    Cow::Borrowed("connection") => connection = Some(query_pair.1),
                    _ => {}
                }
            }

            match device_id {
                Some(name) => {
                    let namespace = ctx.durable_object("DEVICES")?;
                    let id = namespace.id_from_name(&name)?;
                    let stub = id.get_stub()?;

                    // Store the DeviceID -> Connection mapping in KV store
                    if let Some(con) = connection {
                        if let Ok(kv) = ctx.kv(DEVICE_ID_CONNECTION_MAPPING_KV_NAMESPACE) {
                            kv.put(&id.to_string(), con)?.execute().await?;
                        }
                    }

                    stub.fetch_with_request(req).await
                }
                _ => Response::error("Bad Request - missing device id", 400),
            }

        })
        .run(req, env)
        .await
}
