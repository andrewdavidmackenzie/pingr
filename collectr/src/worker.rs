use std::borrow::Cow;
use worker::*;

use data_model::{DeviceDetails, StateChange};

mod device;

const DEVICE_STATUS_KV_NAMESPACE: &str = "DEVICE_STATUS";
const DEVICE_DETAILS_KV_NAMESPACE: &str = "DEVICE_DETAILS";
const CONNECTION_DEVICE_STATUS_KV_NAMESPACE: &str = "CONNECTION_DEVICE_STATUS";

/*
let headers = req.headers();
if let Ok(Some(ip)) = headers.get("CF-Connecting-IP") {
    console_log!("Source IP = {:?}", ip);
}
 */

async fn device_report(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut device_id = None;
    let url = req.url().unwrap();
    for query_pair in url.query_pairs() {
        if let Cow::Borrowed("device_id") = query_pair.0 {
            device_id = Some(query_pair.1)
        }
    }

    match device_id {
        Some(name) => {
            let namespace = ctx.durable_object("DEVICES")?;
            let id = namespace.id_from_name(&name)?;
            let stub = id.get_stub()?;
            stub.fetch_with_request(req).await
        }
        _ => Response::error("Bad Request - missing device_id", 400),
    }
}

#[event(fetch, respond_with_errors)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/report/:type", |req, ctx| async move {
            device_report(req, ctx).await
        })
        .post_async("/report/:type", |req, ctx| async move {
            device_report(req, ctx).await
        })
        .run(req, env)
        .await
}

/*
async fn send_notification_email(to: &str, state_change: &StateChange) -> Result<()> {
    let mut request = Request::new("https://api.mailchannels.net/tx/v1/send", Method::Post)?;
    let mut request_headers = request.headers_mut()?;
    let mut email_headers = Headers::new();
    email_headers.append("content-type", "application/json");
    *request_headers = email_headers;

    /*
    {
        body: JSON.stringify({
            personalizations: [
                { to: [
                    {
                        email: "test@example.com",
                        name: "Test Recipient"
                    }
                    ],
                },
            ],
            from: {
                email: "sender@example.com",
                name: "Workers - MailChannels integration",
            },
            subject: "Look! No servers",
            content: [
                {
                    type: "text/plain",
                    value: "And no email service accounts and all for free too!",
                },
            ],
        }),
    });
    */

    Ok(())
}
 */

// Consume messages from the "state-changes" queue (this worker setup to consume from that queue in Dashboard)
// The device's state is persisted in the `DEVICE_STATUS` KV namespace. This enables picking up
// the previous state between DO invocations, and also exposed the state to other workers and pages
// projects for further processing, notifications, and visualization.
#[event(queue)]
pub async fn main(message_batch: MessageBatch<StateChange>, env: Env, _ctx: Context) -> Result<()> {
    // Deserialize the message batch
    let messages = message_batch.messages()?;

    // Loop through the messages
    for message in messages {
        let state_change: &StateChange = message.body();
        let id = &state_change.id;

        console_log!(
            "Got state-change with message.id: {} state-change: {:?}",
            message.id(),
            state_change,
        );

        let kv = env.kv(DEVICE_STATUS_KV_NAMESPACE)?;
        kv.put(id, state_change)?.execute().await?;

        if let Some(con) = &state_change.connection {
            // Store the Connection::DeviceID -> StateChange in KV store
            let kv = env.kv(CONNECTION_DEVICE_STATUS_KV_NAMESPACE)?;
            let connection_device_key = format!("{}::{}", con, id);
            kv.put(&connection_device_key, state_change)?
                .execute()
                .await?;
        }

        // If the device does not have an entry in the DEVICE_DETAILS table, it's a new device so
        // create a default one - that can then be edited via GUI later.
        let kv = env.kv(DEVICE_DETAILS_KV_NAMESPACE)?;
        if kv.get(id).text().await?.is_none() {
            kv.put(id, DeviceDetails::default())?.execute().await?;
        }
    }

    // Retry all messages
    message_batch.retry_all();
    Ok(())
}
