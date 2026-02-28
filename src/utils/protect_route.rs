use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

pub fn is_authenticated() -> bool {
    window().local_storage().ok().flatten()
        .and_then(|s| s.get_item("accessToken").ok().flatten())
        .is_some()
}