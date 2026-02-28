use leptos::prelude::window;

pub fn get_token() -> String {
    window().local_storage().ok().flatten()
            .and_then(|s| s.get_item("accessToken").ok().flatten())
            .unwrap_or_default()
}