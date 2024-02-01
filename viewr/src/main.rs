mod devices;
mod ssids;
use devices::DeviceStatusList;
use leptos::*;
use leptos_router::*;
use ssids::SSIDStatusList;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=SSIDStatusList/>
                <Route path="/devices" view=DeviceStatusList/>
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}
