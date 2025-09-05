use leptos::*;
use nimbus_ui::App;

fn main() {
    // Initialize panic hook for better error messages in browser
    console_error_panic_hook::set_once();

    // Initialize logger
    wasm_logger::init(wasm_logger::Config::default());

    // Mount the app
    mount_to_body(|| view! { <App/> });
}
