use crate::pages::fetch_nearest_stations_dto::{
    DiscountCodeResponse, Station, fetch_closests, generate_discount_code,
};
use crate::utils::get_stations_imgs::G_STATION_IMAGES;
use crate::utils::get_gps_location::locate;
use crate::utils::validate_boundary;
use leptos::{logging, prelude::*};

#[component]
pub fn Home_Gas() -> impl IntoView {
    let get_stations_action = Action::new_local(move |_: &()| {
        async move {
            if let Ok((lat, lon)) = locate().await {
                logging::log!("these are the lat an lon {}, {}", lat, lon);
                let _ = validate_boundary::validate_abuja_bounds(lat, lon)?;
                fetch_closests(lat, lon, "gas".to_string()).await
            } else {
                Err("GPS took too long or permission was denied.".to_string())
            }
        }
    });

    let stations_result = get_stations_action.value();
    let selected_station = RwSignal::new(None::<Station>);
    let discount_code_action = Action::new_local(move |station_id: &String| {
        let station_id = station_id.clone();
        async move { generate_discount_code(station_id).await }
    });
    let details_ref = NodeRef::<leptos::html::Div>::new();
    let has_scrolled = RwSignal::new(false);

    Effect::new(move |_| {
        if selected_station.get().is_some() && !has_scrolled.get() {
            if let Some(el) = details_ref.get() {
                let win = web_sys::window().unwrap();
                let width = win.inner_width().ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1024.0);
                if width <= 640.0 {
                    el.scroll_into_view_with_bool(true);
                    has_scrolled.set(true);
                }
            }
        }
    });

    view! {
        <div class="fuel-page gas-theme">
            <div class=move || match stations_result.get() {
                Some(Ok(ref s)) if !s.is_empty() => "fuel-hero fuel-hero--stations-loaded",
                _ => "fuel-hero",
            }>
                <img class="fuel-hero-logo" src="assets/gas_cylinder/cylinder3.jpeg" alt="Gas Station"  />
            </div>
            
            <button 
                class="fuel-locate-button" 
                disabled=move || get_stations_action.pending().get()
                on:click=move |_| { get_stations_action.dispatch(()); }
            >
                {move || if get_stations_action.pending().get() { 
                    "Finding..." 
                } else { 
                    "Find Gas Station" 
                }}
            </button>

            <div class="fuel-content-grid">
                <div class="fuel-results">
                    {move || match stations_result.get() {
                        Some(Ok(stations)) => {
                            if stations.is_empty() {
                                view! { <p class="status-msg">"No stations found in your area."</p> }.into_any()
                            } else {
                                let grid_class = if stations.len() > 2 {
                                    "fuel-station-grid fuel-station-grid--mobile-two"
                                } else {
                                    "fuel-station-grid"
                                };
                                view! { 
                                    <>
                                        <p class="fuel-results-hint">"Click a station to view details below"</p>
                                        <ul class=grid_class>
                                            {stations.into_iter().enumerate().map(|(i, s)|{
                                                // FIX 1: Use modulo (%) to prevent index out of bounds
                                                let image_url = G_STATION_IMAGES[i % G_STATION_IMAGES.len()];
                                                let station = s.clone();
                                                let station_id = s.id.clone();
                                                
                                                view! { 
                                                    <li 
                                                        class=move || if selected_station.get().as_ref().map(|sel| sel.id == station_id).unwrap_or(false) 
                                                            {"fuel-station-card is-selected"} else {"fuel-station-card"}
                                                        on:click=move |_| selected_station.set(Some(station.clone()))
                                                    >
                                                        <img src=image_url class="fuel-station-image" alt="Station" />
                                                        <div class="fuel-station-name">
                                                            <p>{s.name}</p> 
                                                        </div>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    </>
                                }.into_any()
                            }
                        },
                        Some(Err(e)) => {
                            if e.contains("Geolocation failed") || e.contains("GPS took too long or permission was denied") {
                                view! {
                                    <p class="error-msg">
                                        "Something went wrong! It could be that location access was denied."<br/>
                                        "Please make sure location access is **enabled** in your browser settings and retry."
                                    </p>
                                }.into_any()
                            }else if e.contains("outside the Abuja service area") {
                                 view! { <p class="error-msg">{format!("Oops! {e}")}</p> }.into_any()
                            } 
                            else {
                                view! { <p class="error-msg">{format!("Oops! something went wrong, give some seconds, refresh and retry")}</p> }.into_any()
                            }
                        },
                        None => view! { <p class="status-msg">"NB: Allow this app use your current location when prompted (" <span class="status-msg-region">"service currently available only in Abuja"</span>")"</p> }.into_any(),
                    }}
                </div>

                <div class="fuel-details-card" node_ref=details_ref>
                    {move || match selected_station.get() {
                        Some(s) => {
                            let map_url = format!("https://www.google.com/maps/search/?api=1&query={},{}", s.latitude, s.longitude);
                            
                            // FIX 2: Safely access commodities
                            let selected_station_id = s.id.clone();
                            let selected_station_name = s.name.clone();
                            let first_commodity = s.commodities.first().cloned();

                            let price = first_commodity.as_ref()
                                .map(|c| format!("{}", c.price))
                                .unwrap_or_else(|| "N/A".to_string());

                            let is_discount_enabled = first_commodity
                                .as_ref()
                                .and_then(|c| c.discount_enabled)
                                .unwrap_or(false);
                            let discount_percentage = first_commodity
                                .as_ref()
                                .and_then(|c| c.discount_percentage)
                                .unwrap_or(0);

                            let discount_result = discount_code_action
                                .value()
                                .get()
                                .and_then(|res| {
                                    res.ok().filter(|code: &DiscountCodeResponse| {
                                        code.code
                                            .to_ascii_uppercase()
                                            .starts_with(&selected_station_name.to_ascii_uppercase().chars().filter(|c| c.is_ascii_alphanumeric()).take(2).collect::<String>())
                                    })
                                });

                            view! {
                                <div class="fuel-details-content">
                                    <h2>{s.name}</h2>
                                    <div class="fuel-info-section">
                                        <div class="fuel-info-item"><strong>"Price(₦): "</strong> {price}</div>
                                        {if is_discount_enabled {
                                            view! {
                                                <div class="fuel-info-item" style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
                                                    <div>
                                                        <strong>"Discount: "</strong>
                                                        {format!("{}% available", discount_percentage)}
                                                    </div>
                                                    <button
                                                        class="fuel-locate-button"
                                                        style="width: auto; margin-top: 0;"
                                                        disabled=move || discount_code_action.pending().get()
                                                        on:click=move |_| {
                                                            discount_code_action.dispatch(selected_station_id.clone());
                                                        }
                                                    >
                                                        {move || if discount_code_action.pending().get() {
                                                            "Generating code..."
                                                        } else {
                                                            "Generate Discount Code"
                                                        }}
                                                    </button>

                                                    {move || discount_result.clone().map(|code| view! {
                                                        <div class="fuel-info-item" style="width: 100%; margin-top: 8px; background: #f6fff7; padding: 10px; border-radius: 8px;">
                                                            <div><strong>"Code: "</strong>{code.code.clone()}</div>
                                                            <div style="margin-top: 6px;">"Present this code to redeem your discount, code expires in 24hrs"</div>
                                                        </div>
                                                    })}

                                                    {move || discount_code_action.value().get().and_then(|res| res.err()).map(|err| view! {
                                                        <small class="error-message" style="width: 100%; margin-top: 8px;">{err}</small>
                                                    })}
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <></> }.into_any()
                                        }}
                                        <div class="fuel-info-item"><strong>"Address: "</strong> {s.address}</div>
                                        <div class="fuel-info-item">
                                            <strong>"Directions: "</strong>
                                            <a href=map_url target="_blank" rel="noopener noreferrer" class="fuel-station-direction">
                                                "Open Google Maps"
                                            </a>
                                        </div>
                                        <div class="fuel-info-item"><strong>"Distance: "</strong> {format!("{:.2}km", s.distance.unwrap_or_else(|| 0.0))}</div>
                                    </div>

                                </div>
                            }.into_any()
                        },
                        None => view! { <div class="fuel-details-empty">"Select a station to view details"</div> }.into_any(),
                    }}
                </div>
            </div>
        </div>
    }
}