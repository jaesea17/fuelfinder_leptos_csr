use gloo_net::http::Request;
use leptos::prelude::*;
use crate::{
    pages::fetch_nearest_stations_dto::Commodity,
    pages::stations::dashboard::utils::get_token,
    utils::base_url::BaseUrl,
};

#[component]
pub fn CommodityCard(
    commodity: Commodity,
    station_type: String,
) -> impl IntoView {
    let is_editing = RwSignal::new(false);
    let price_input = RwSignal::new(commodity.price.to_string());
    let error_msg = RwSignal::new(None::<String>);

    let stored_id = StoredValue::new(commodity.id.clone());
    let current_price = RwSignal::new(commodity.price);
    let commodity_name = commodity.name.clone();
    let is_available = RwSignal::new(commodity.is_available);
    let previous_availability = RwSignal::new(commodity.is_available);
    let previous_price = RwSignal::new(commodity.price);

    let unit = if station_type.to_lowercase().contains("gas") { "KG" } else { "Litre" };
    let title_text = format!("{} - Price/{}", commodity_name.to_uppercase(), unit);

    // Price update — owned by this card; on success update local signal only
    let update_action: Action<(String, i32), Result<i32, String>> = Action::new_local(move |(id, new_price): &(String, i32)| {
        let id = id.clone();
        let price = *new_price;
        async move {
            let token = get_token();
            let base_url = BaseUrl::get_base_url();
            let url = format!("{base_url}/api/v1/commodities/{}", id);
            let status = price > 0;
            let body = serde_json::json!({ "price": price, "is_available": status });

            Request::patch(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .json(&body)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            Ok(price)
        }
    });

    // On save success: update displayed price; if zero, mark unavailable
    Effect::new(move |_| {
        if let Some(Ok(new_price)) = update_action.value().get() {
            current_price.set(new_price);
            error_msg.set(None);
            if new_price == 0 {
                is_available.set(false);
            } else {
                is_available.set(true);
            }
        }
    });

    let is_updating = move || update_action.pending().get();

    // Availability toggle — optimistic; rollback + show error on failure
    let toggle_action: Action<(String, bool, i32), Result<(), String>> = Action::new_local(move |(id, status, price): &(String, bool, i32)| {
        let id = id.clone();
        let status = *status;
        let price = *price;
        async move {
            let token = get_token();
            let base_url = BaseUrl::get_base_url();
            let url = format!("{base_url}/api/v1/commodities/{}", id);
            let body = serde_json::json!({ "price": price, "is_available": status });

            Request::patch(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .json(&body)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            Ok(())
        }
    });

    // Rollback availability if toggle fails
    Effect::new(move |_| {
        if let Some(Err(e)) = toggle_action.value().get() {
            is_available.set(previous_availability.get());
            current_price.set(previous_price.get());
            price_input.set(previous_price.get().to_string());
            error_msg.set(Some(format!("Toggle failed: {}", e)));
        }
    });

    let is_toggling = move || toggle_action.pending().get();

    view! {
        <div class="station-card">
            <div class="card-header">
                <h2>{title_text}</h2>

                <Show
                    when=move || is_editing.get()
                    fallback=move || {
                        view! {
                            <div class="price-display-wrapper">
                                {move || if is_updating() || is_toggling() {
                                    view! { <div class="price-spinner"></div> }.into_any()
                                } else {
                                    view! {
                                        <div><p class="price">"₦" {move || current_price.get().to_string()}</p></div>
                                    }.into_any()
                                }}
                            </div>
                            <button
                                class="edit-button"
                                disabled=move || is_updating()
                                on:click=move |_| is_editing.set(true)
                            >
                                "Update"
                            </button>
                        }
                    }
                >
                    <div class="edit-section">
                        <input
                            type="number"
                            class="price-input"
                            prop:value=move || price_input.get()
                            on:input=move |ev| price_input.set(event_target_value(&ev))
                        />
                        <button class="save-button"
                            on:click=move |_| {
                                if let Ok(new_val) = price_input.get().parse::<i32>() {
                                    update_action.dispatch((stored_id.get_value(), new_val));
                                    is_editing.set(false);
                                }
                            }
                        >
                            "Save"
                        </button>
                        <button class="cancel-button" on:click=move |_| is_editing.set(false)>
                            "Cancel"
                        </button>
                    </div>
                </Show>
            </div>
            <div class="card-body">
                <div class="availability-status">
                    <span class=move || format!("status-dot {}", if is_available.get() { "available" } else { "unavailable" })></span>
                    <span>{move || if is_available.get() { "Available" } else { "Not Available" }}</span>
                </div>

                <div class="availability-toggle">
                    <label class="switch">
                        <input
                            type="checkbox"
                            prop:checked=move || is_available.get()
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                if checked && current_price.get() <= 0 {
                                    error_msg.set(Some("Please update price first".into()));
                                    is_available.set(false);
                                } else {
                                    previous_availability.set(is_available.get());
                                    previous_price.set(current_price.get());

                                    is_available.set(checked);

                                    let next_price = if checked {
                                        current_price.get()
                                    } else {
                                        current_price.set(0);
                                        price_input.set("0".to_string());
                                        0
                                    };

                                    toggle_action.dispatch((stored_id.get_value(), checked, next_price));
                                    error_msg.set(None);
                                }
                            }
                        />
                        <span class="slider"></span>
                    </label>
                </div>
            </div>
            {move || error_msg.get().map(|err| view! { <p class="error-message">{err}</p> })}
        </div>
    }
}