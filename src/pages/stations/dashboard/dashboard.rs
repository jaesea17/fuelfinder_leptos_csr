use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::pages::fetch_nearest_stations_dto::Station;
// Adjust this import path to where your CommodityCard is located
use crate::pages::stations::dashboard::commodity_card::CommodityCard;
use crate::pages::stations::dashboard::utils::get_token;
use crate::utils::base_url::BaseUrl;

#[component]
pub fn StationDashboard() -> impl IntoView {
    let navigate = use_navigate();

    // LocalResource handles browser-only types (like localStorage) safely
    let station_resource = LocalResource::new(|| async move {
        let token = get_token();
        let BASE_URL = BaseUrl::get_base_url();
        let url = format!("{BASE_URL}/api/v1/stations/dashboard");
        let resp = Request::get(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        resp.json::<Station>().await.map_err(|e| e.to_string())
    });

    // Action for updating prices - remains local for WASM compatibility
    let update_price_action = Action::new_local(move |(id, new_price): &(String, i32)| {
        let id = id.clone();
        let price = *new_price;
        let mut status = true;
        if price == 0 {status = false;}

        async move {
            let BASE_URL = BaseUrl::get_base_url();
            let url = format!("{BASE_URL}/api/v1/commodities/{}", id);
            let body = serde_json::json!({ "price": price, "is_available": status });
            let token = get_token();
            
            let _ = Request::patch(&url)
                .header("Authorization", &format!("Bearer {token}"))
                .json(&body)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;
            
            // Refetch the data so the UI updates with the new server state
            station_resource.refetch(); 
            Ok(())
        }
    });

    view! {
        <div class="station-dashboard">
            <Suspense fallback=move || view! { <p class="loading">"Loading dashboard data..."</p> }>
                {move || station_resource.get().map(|res| match res {
                    Ok(data) => view! {
                        <h1>{data.name}</h1>
                        <div class="commodities-grid">
                            <For
                                each=move || data.commodities.clone()
                                key=|c| c.id.clone()
                                children=move |commodity| {
                                    view! { 
                                        <CommodityCard 
                                            commodity=commodity 
                                            update_action=update_price_action
                                            station_resource=station_resource 
                                        /> 
                                    }
                                }
                            />
                        </div>
                    }.into_any(),
                    Err(_) => {
                        navigate("/signin", Default::default());
                        view! { <p>"Unauthorized - Redirecting..."</p> }.into_any()
                    }
                })}
            </Suspense>
        </div>
    }
}