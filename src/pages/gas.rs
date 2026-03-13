use crate::pages::fetch_nearest_stations_dto::{Station, fetch_closests};
use crate::utils::get_stations_imgs::STATION_IMAGES;
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

    view! {
        <div class="fuel-page gas-theme">
            <div class="fuel-hero">
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
                                    <ul class=grid_class>
                                        {stations.into_iter().enumerate().map(|(i, s)|{
                                            // FIX 1: Use modulo (%) to prevent index out of bounds
                                            let image_url = STATION_IMAGES[i % STATION_IMAGES.len()];
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
                                view! { <p class="error-msg">{format!("Oops! something went wrong")}</p> }.into_any()
                            }
                        },
                        None => view! { <p class="status-msg">"Stations will appear here (service currently available only in Abuja)"</p> }.into_any(),
                    }}
                </div>

                <div class="fuel-details-card">
                    {move || match selected_station.get() {
                        Some(s) => {
                            let map_url = format!("https://www.google.com/maps/search/?api=1&query={},{}", s.latitude, s.longitude);
                            
                            // FIX 2: Safely access commodities
                            let price = s.commodities.first()
                                .map(|c| format!("{}", c.price))
                                .unwrap_or_else(|| "N/A".to_string());

                            view! {
                                <div class="fuel-details-content">
                                    <h2>{s.name}</h2>
                                    <div class="fuel-info-section">
                                        <div class="fuel-info-item"><strong>"Price(₦): "</strong> {price}</div>
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