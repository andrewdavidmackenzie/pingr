use leptos::*;
use leptos_router::*;

use connections::ConnectionDeviceStatusList;
use devices::DeviceStatusList;

mod connections;
mod devices;

#[component]
#[allow(non_snake_case)]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=ConnectionDeviceStatusList/>
                <Route path="/devices" view=DeviceStatusList/>
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
