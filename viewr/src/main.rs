pub mod devices;
use devices::DeviceStatusList;
use leptos::*;
use leptos_router::*;

#[component]
fn Home() -> impl IntoView {
    view! {
        <h1>HOME PAGE</h1>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/devices" view=DeviceStatusList/>
            </Routes>
        </Router>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| view! { <App/> })
}
