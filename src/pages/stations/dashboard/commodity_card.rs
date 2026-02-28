use gloo_net::http::Request;
use leptos::prelude::*;
use crate::{pages::{fetch_nearest_stations_dto::{Commodity, Station}, stations::dashboard::utils::get_token}, 
utils::base_url::BaseUrl};

#[component]
pub fn CommodityCard(
    commodity: Commodity, 
    update_action: Action<(String, i32), Result<(), String>>,
    station_resource: LocalResource<Result<Station, String>>
) -> impl IntoView {
    let is_editing = RwSignal::new(false);
    let price_input = RwSignal::new(commodity.price.to_string());
    let error_msg = RwSignal::new(None::<String>);
 
    //  Extract static data and Store these so they are "Copy" and accessible in any closure below
    let stored_id = StoredValue::new(commodity.id.clone());
    let stored_price = StoredValue::new(commodity.price);
    let commodity_name = commodity.name.clone();
    let is_available = RwSignal::new(commodity.is_available);

    // Derived signal for the spinner
    let is_updating_this = move || {
        update_action.pending().get() && 
        update_action.input().get().map(|(id, _)| id == stored_id.get_value()).unwrap_or(false)
    };

    let toggle_action: Action<(String, bool), Result<(), String>> = Action::new_local(move |(id, status): &(String, bool)| {
        let id = id.clone();
        let status = *status;
        async move {
            let token = get_token();
            let base_url = BaseUrl::get_base_url();
            let url = format!("{base_url}/api/v1/commodities/{}", id);
            let body = serde_json::json!({ "price": 0, "isAvailable": status });
            
            Request::patch(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .json(&body)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;
        
            station_resource.refetch(); 
            Ok(())
        }
    });
    let is_toggling_this = move || {
        toggle_action.pending().get() && 
        toggle_action.input().get().map(|(id, _)| id == stored_id.get_value()).unwrap_or(false)
    };

    view! {
        <div class="station-card">
            <div class="card-header">
                <h2>{move || commodity_name.to_uppercase()} " - Price/Litre"</h2>
                
                <Show 
                    when=move || is_editing.get()
                    fallback=move || {
                        // Accessing stored_id and stored_price here is safe and repeated
                        view! {
                            <div class="price-display-wrapper">
                                {move || if is_updating_this() || is_toggling_this() {
                                    view! { <div class="price-spinner"></div> }.into_any()
                                } else {
                                    view! { 
                                        <div><p class="price">"₦" {stored_price.get_value().to_string()}</p></div> 
                                    }.into_any()
                                }}
                            </div>
                            <button 
                                class="edit-button" 
                                disabled=move || is_updating_this()
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
                                if checked && stored_price.get_value() <= 0 {
                                    error_msg.set(Some("Please update price first".into()));
                                    is_available.set(false);
                                } else {
                                    is_available.set(checked);
                                    toggle_action.dispatch((stored_id.get_value(), checked));
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