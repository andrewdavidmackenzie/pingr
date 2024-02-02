use leptos::{error::Result, *};
use reqwasm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Device {
    name: String,
}

async fn api_device_list() -> Result<Vec<String>> {
    let res = reqwasm::http::Request::get("/api/device/list")
        .send()
        .await?
        .json::<Vec<Device>>()
        .await?
        .into_iter()
        .map(|device| device.name)
        .collect::<Vec<String>>();
    Ok(res)
}

#[component]
#[allow(non_snake_case)]
fn DeviceList() -> impl IntoView {
    let device_list = create_local_resource(move || (), |_| api_device_list());

    view! {
        <h1>"Device Status"</h1> {
            move || match device_list.get() {
                None => view! { <p>"Searching for devices..."</p> }.into_view(),
                Some(Ok(devices)) => view! {
                    <ul>
                        {devices.into_iter()
                            .map(|device| view! {<li>{device}</li>})
                            .collect_view()}
                    </ul>
                }.into_view(),
                Some(Err(_)) => view! {<p>"Error finding devices"</p>}.into_view(),
            }
        }
        <button on:click=move |_| { device_list.refetch() }>
            "Refresh"
        </button>
    }
}

async fn api_device_status_get(key: &str) -> Result<String> {
    let res = reqwasm::http::Request::get(&format!("/api/device/status/{key}"))
        .send()
        .await?
        .json::<String>()
        .await?;
    Ok(res)
}

async fn api_device_status_list() -> Result<Vec<(String, String)>> {
    let device_ids = api_device_list().await?;

    let mut statuses = vec![];

    for device_id in device_ids {
        statuses.push((
            device_id.clone(),
            api_device_status_get(&device_id.to_string()).await?,
        ));
    }

    Ok(statuses)
}

#[component]
#[allow(non_snake_case)]
pub fn DeviceStatusList() -> impl IntoView {
    let device_statuses = create_local_resource(move || (), |_| api_device_status_list());

    view! {
        <h1>"Device Status"</h1> {
            move || match device_statuses.get() {
                None => view!{ <p>"Searching for devices..."</p> }.into_view(),
                Some(Ok(devices)) => {
                    if devices.is_empty() {
                        view!{ <p>No devices found</p> }.into_view()
                    } else {
                        // TODO move all this into the api method
                        let mut status_map = HashMap::<&str, Vec<&String>>::new();
                        for (device_id, device_status) in &devices {
                            status_map.entry(device_status)
                                .or_insert_with(Vec::new)
                                .push(device_id);
                        }
                        ["Reporting", "NotReporting", "Stopped"].map(|status| {
                            match status_map.get(status) {
                                Some(id_list) => {
                                    view!{
                                        <ul>{status}
                                            {
                                                id_list.into_iter()
                                                    .map(|device_id| view! {<li>{device_id.to_string()}</li>})
                                                    .collect_view()
                                            }
                                        </ul>
                                    }.into_view()
                                },
                                None => {
                                    view!{ <ul>{status}</ul>}.into_view()
                                }
                            }
                        }).collect_view()
                    }
                },
                Some(Err(_)) => view! {<p>"Error finding devices"</p>}.into_view(),
            }
        }
        <button on:click=move |_| { device_statuses.refetch() }>
            "Refresh"
        </button>
    }
}
