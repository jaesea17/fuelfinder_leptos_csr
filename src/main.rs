use leptos::prelude::*;
use fuelfinder_client::App;
use wasm_bindgen::JsCast; // Required for type casting
use web_sys::HtmlElement; // The specific type Leptos wants

fn main() {
    // set up logging
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    let document = document();
    // 1. Find the loading screen specifically
    if let Some(loader) = document.get_element_by_id("loading-screen") {
        loader.remove(); // Manually kill the spinner
    }
    // Find the Initial spinner HtmlElement to you can replace it with our App
    let spinner = document
        .get_element_by_id("app")
        .expect("element with id 'app' not found")
        // Cast generic 'Element' into 'HtmlElement'
        .unchecked_into::<HtmlElement>();

    mount_to(spinner, || view! { <App /> }).forget();
}