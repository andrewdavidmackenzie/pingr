use leptos::{error::Result, *};
use reqwasm;
use serde::{Deserialize, Serialize};

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
        .map(|device| device.name.to_string())
        .collect::<Vec<_>>();
    Ok(res)
}

#[component]
fn DeviceList() -> impl IntoView {
    let device_list = create_local_resource(move || (), |_| async move { api_device_list().await });

    view! {
        <h1>"Devices"</h1>
        {move || match device_list.get() {
            None => view! { <p>"Searching for devices..."</p> }.into_view(),
            Some(devices) => view! {
                <ul>
                    {devices.into_iter()
                        .map(|device| view! {<li>{device}</li>})
                        .collect_view()}
                </ul>
            }.into_view()
        }}
        <button on:click=move |_| { device_list.refetch() }>
            "Refresh"
        </button>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <DeviceList/> })
}
