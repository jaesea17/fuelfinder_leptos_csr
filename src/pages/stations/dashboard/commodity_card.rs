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
    
    // 1. Extract the data needed for the logic
    let commodity_id = commodity.id.clone();
    let commodity_name = commodity.name.clone();
    let initial_price = commodity.price;
    let is_available = RwSignal::new(commodity.is_available);

    // 2. Wrap the logic in a StoredValue. 
    // This makes the closure "Copy" so it can be used in the view multiple times.
    let handle_save = StoredValue::new(move |_| {
        match price_input.get().parse::<i32>() {
            Ok(new_val) => {
                update_action.dispatch((commodity_id.clone(), new_val));
                is_editing.set(false);
                error_msg.set(None);
            },
            Err(_) => error_msg.set(Some("Invalid price number".into())),
        }
    });

    let toggle_action: Action<(String, bool), Result<(), String>> = Action::new_local(move |(id, status): &(String, bool)| {
        let id = id.clone();
        let status = *status;
        let price = 0;
        async move {
            let token = get_token();
            let BASE_URL = BaseUrl::get_base_url();
            let url = format!("{BASE_URL}/api/v1/commodities/{}", id);
            let body = serde_json::json!({ "price": price, "isAvailable": status });
            
            Request::patch(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .json(&body)
            .map_err(|e| e.to_string())?
            .send().await.map_err(|e| e.to_string())?;
        
            station_resource.refetch(); 
            Ok(())
        }
    });
    view! {
        <div class="station-card">
            <div class="card-header">
                <h2>{move || commodity_name.to_uppercase()} " - Price/Litre"</h2>
                
                <Show 
                    when=move || is_editing.get()
                    fallback=move || view! {
                        <p class="price">"₦" {initial_price.to_string()}</p>
                        <button class="edit-button" on:click=move |_| is_editing.set(true)>
                            "Update"
                        </button>
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
                            // Use .with_value() to call the stored closure
                            on:click=move |ev| handle_save.with_value(|f| f(ev))
                        >
                            {move || if update_action.pending().get() { "Saving..." } else { "Save" }}
                        </button>
                        <button class="cancel-button" on:click=move |_| is_editing.set(false)>
                            "Cancel"
                        </button>
                    </div>
                </Show>
            </div>
            <div class="card-body">
                <div class="availability-status">
                    // Status dot changes color based on the reactive signal
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
                                
                                // Logic: If trying to turn ON, check if price is valid
                                if checked && initial_price <= 0 {
                                    error_msg.set(Some("Please update price first".into()));
                                    // Reset the checkbox visually
                                    is_available.set(false);
                                } else {
                                    is_available.set(checked);
                                    toggle_action.dispatch((commodity.id.clone(), checked));
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