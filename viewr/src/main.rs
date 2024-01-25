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
        .map(|device| device.name)
        .collect::<Vec<String>>();
    logging::log!("device list {:?}", res);
    Ok(res)
}

#[component]
fn DeviceList() -> impl IntoView {
    let device_list = create_local_resource(move || (), |_| api_device_list());

    view! {
        <h1>"Devices"</h1>
        <ul> {
            move || match device_list.get() {
                None => view! { <p>"Searching for devices..."</p> }.into_view(),
                Some(Ok(devices)) => view! {
                        {devices.into_iter()
                            .map(|device| view! {<li>{device}</li>})
                            .collect_view()}
                }.into_view(),
                Some(Err(_)) => view! {<p>"Error finding devices"</p>}.into_view(),
            }
        }
        </ul>
        <button on:click=move |_| { device_list.refetch() }>
            "Refresh"
        </button>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <DeviceList/> })
}
