use leptos::{error::Result, *};
use reqwasm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ConnectionDevice {
    name: String,
}

async fn api_connection_device_list() -> Result<Vec<String>> {
    let res = reqwasm::http::Request::get("/api/connection/list")
        .send()
        .await?
        .json::<Vec<ConnectionDevice>>()
        .await?
        .into_iter()
        .map(|connection_device| connection_device.name)
        .collect::<Vec<String>>();
    Ok(res)
}

async fn api_connection_device_status_list() -> Result<HashMap<String, Vec<(String, String)>>> {
    let connection_device_ids = api_connection_device_list().await?;

    let mut statuses = HashMap::<String, Vec<(String, String)>>::new();

    // TODO async request to get that connection_device's status
    for connection_device_id in connection_device_ids {
        if let Some((connection, device)) = connection_device_id.split_once("::") {
            statuses
                .entry(connection.to_string())
                .or_insert_with(Vec::new)
                .push((device.to_string(), "Reporting".to_string()));
        }
    }

    Ok(statuses)
}

#[component]
#[allow(non_snake_case)]
pub fn ConnectionDeviceStatusList() -> impl IntoView {
    let connection_device_statuses =
        create_local_resource(move || (), |_| api_connection_device_status_list());

    view! {
        <h1>"Connections | Devices"</h1>
        {
            move || match connection_device_statuses.get() {
                None => view!{ <p>"Searching for devices..."</p> }.into_view(),
                Some(Ok(connection_device_statuses)) => {
                    if connection_device_statuses.is_empty() {
                        view!{ <p>No connections found</p> }.into_view()
                    } else {
                        view!{
                            <ul> {
                                connection_device_statuses.iter().map(|(connection, device_status_list)| {
                                    //let (_connection_type, connection_name) = connection.split_once("=").unwrap();
                                    view! {
                                        <li>
                                            {connection}
                                            {
                                                device_status_list.iter().map(|(device_id, status)| {
                                                    let status_style = format!("tooltip device-status {status}");
                                                    view!{
                                                        <div class=status_style>DEV
                                                            <span class="tooltiptext">{device_id}</span>
                                                        </div>}
                                                }).collect_view()
                                            }
                                        </li>
                                    }
                                }).collect_view()
                            } </ul>
                        }.into_view()
                    }
                },
                Some(Err(_)) => view! {<p>"Error finding connections"</p>}.into_view(),
            }
        }
        <button on:click=move |_| { connection_device_statuses.refetch() }>
            "Refresh"
        </button>
    }
}
