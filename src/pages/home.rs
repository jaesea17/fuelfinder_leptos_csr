use crate::pages::fetch_nearest_stations_dto::{Station, fetch_closests};
use crate::utils::get_stations_imgs::STATION_IMAGES;
use crate::utils::get_gps_location::locate;
use crate::utils::validate_boundary;
use leptos::{logging, prelude::*};

#[component]
pub fn Home() -> impl IntoView {
    let get_stations_action = Action::new_local(move |_: &()| {
        async move {
            if let Some((lat, lon)) = locate().await {
                logging::log!("these are the lat an lon {}, {}", lat, lon);
                let _ = validate_boundary::validate_abuja_bounds(lat, lon)?;
                fetch_closests(lat, lon).await
            } else {
                Err("GPS took too long or permission was denied.".to_string())
            }
        }
    });

    let stations_result = get_stations_action.value();
    let selected_station = RwSignal::new(None::<Station>);

    view! {
        <div class="home-container">
            <button 
                class="locate-button" 
                disabled=move || get_stations_action.pending().get()
                on:click=move |_| { get_stations_action.dispatch(()); }
            >
                {move || if get_stations_action.pending().get() { 
                    "Finding..." 
                } else { 
                    "Find Fuel" 
                }}
            </button>

            <div class="results-container">
                {move || match stations_result.get() {
                    Some(Ok(stations)) => {
                        if stations.is_empty() {
                            view! { <p class="status-msg">"No stations found in your area."</p> }.into_any()
                        } else {
                            view! { 
                                <ul class="dashboard">
                                    {stations.into_iter().enumerate().map(|(i, s)|{
                                        // FIX 1: Use modulo (%) to prevent index out of bounds
                                        let image_url = STATION_IMAGES[i % STATION_IMAGES.len()];
                                        let station = s.clone();
                                        let station_id = s.id.clone();
                                        
                                        view! { 
                                            <li 
                                                class=move || if selected_station.get().as_ref().map(|sel| sel.id == station_id).unwrap_or(false) 
                                                    {"card is-selected"} else {"card"}
                                                on:click=move |_| selected_station.set(Some(station.clone()))
                                            >
                                                <img src=image_url class="card-image" alt="Station" />
                                                <div class="station-name">
                                                    <p>{s.name}</p> 
                                                </div>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }
                    },
                    Some(Err(e)) => view! { <p class="error-msg">"Oops! something went wrong "</p> }.into_any(),
                    None => view! { <p class="status-msg">"Stations will appear here (service currently available only in Abuja)"</p> }.into_any(),
                }}
            </div>

            <div class="details-card">
                {move || match selected_station.get() {
                    Some(s) => {
                        let map_url = format!("https://www.google.com/maps/search/?api=1&query={},{}", s.latitude, s.longitude);
                        
                        // FIX 2: Safely access commodities
                        let price = s.commodities.first()
                            .map(|c| format!("{}", c.price))
                            .unwrap_or_else(|| "N/A".to_string());

                        view! {
                            <div class="details-content">
                                <h2>{s.name}</h2>
                                <div class="info-section">
                                    <div class="info-item"><strong>"Price(₦): "</strong> {price}</div>
                                    <div class="info-item"><strong>"Address: "</strong> {s.address}</div>
                                    <div class="info-item">
                                        <strong>"Directions: "</strong>
                                        <a href=map_url target="_blank" rel="noopener noreferrer" class="station-direction">
                                            "Open Google Maps"
                                        </a>
                                    </div>
                                    <div class="info-item"><strong>"Distance: "</strong> {format!("{:.2}km", s.distance.unwrap_or_else(|| 0.0))}</div>
                                </div>
                            </div>
                        }.into_any()
                    },
                    None => view! { <div class="details-empty">""</div> }.into_any(),
                }}
            </div>
        </div>
    }
}