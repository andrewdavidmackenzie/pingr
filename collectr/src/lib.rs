use crate::device::StateChange;
use std::borrow::Cow;
use worker::*;

mod device;

const DEVICE_STATUS_KV_NAMESPACE: &str = "DEVICE_STATUS";
const CONNECTION_DEVICE_STATUS_KV_NAMESPACE: &str = "CONNECTION_DEVICE_STATUS";
const DEVICE_ID_CONNECTION_MAPPING_KV_NAMESPACE: &str = "DEVICE_ID_CONNECTION_MAPPING";

/*
let headers = req.headers();
if let Ok(Some(ip)) = headers.get("CF-Connecting-IP") {
    console_log!("Source IP = {:?}", ip);
}
 */

async fn do_request(req: Request, ctx: RouteContext<()>) -> Result<Response> {
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

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/report/:type", |req, ctx| async move {
            do_request(req, ctx).await
        })
        .post_async("/report/:type", |req, ctx| async move {
            do_request(req, ctx).await
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
        // Log the message and meta data
        console_log!(
            "Got message {:?}, with id {} and timestamp: {}",
            message.body,
            message.id,
            message.timestamp.to_string()
        );

        let state_change: StateChange = message.body;

        // Store the device state in KV store that can be read elsewhere
        let kv = env.kv(DEVICE_STATUS_KV_NAMESPACE)?;
        kv.put(&state_change.id, &state_change.new_state)?
            .execute()
            .await?;

        let kv = env.kv(DEVICE_ID_CONNECTION_MAPPING_KV_NAMESPACE)?;

        let con = match state_change.connection {
            None => {
                // If no connection is supplied - try and find one in the DeviceID to Connections table
                let c = kv.get(&state_change.id).text().await?;
                // and if none there then we can't get it so default to "Unknown"
                c.unwrap_or("Unknown".to_string())
            }
            Some(c) => {
                // Store the DeviceID -> Connection mapping in KV store
                kv.put(&state_change.id, c.to_string())?.execute().await?;
                c
            }
        };

        // Store the Connection::DeviceID -> status in KV store
        let kv = env.kv(CONNECTION_DEVICE_STATUS_KV_NAMESPACE)?;
        kv.put(
            &format!("{}::{}", con, state_change.id),
            state_change.new_state,
        )?
        .execute()
        .await?;
    }

    // Retry all messages
    message_batch.retry_all();
    Ok(())
}
