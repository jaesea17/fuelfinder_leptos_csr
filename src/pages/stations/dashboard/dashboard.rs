use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::pages::fetch_nearest_stations_dto::Station;
use crate::pages::stations::dashboard::commodity_card::CommodityCard;
use crate::pages::stations::dashboard::utils::get_token;
use crate::pages::stations::dto::fetch_station_notifications;
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

    // Separate resource for notifications - non-blocking
    let notifications_resource = LocalResource::new(|| async move {
        let token = get_token();
        fetch_station_notifications(token).await
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
            // Notification banner — renders as soon as ready, never blocks station content
            {move || notifications_resource.get().map(|res| match res {
                Ok(notifs) if !notifs.is_empty() => {
                    let unread_count = notifs.iter().filter(|n| !n.is_read).count();
                    view! {
                        <div class="notifications-panel">
                            <div class="notifications-header">
                                <span class="bell-icon">"🔔"</span>
                                <strong>
                                    {if unread_count > 0 {
                                        format!("Notifications  ({} unread)", unread_count)
                                    } else {
                                        "Notifications".to_string()
                                    }}
                                </strong>
                            </div>
                            <div class="notifications-list">
                                <For
                                    each=move || notifs.clone()
                                    key=|n| n.id.clone()
                                    children=move |notif| {
                                        let kind_class = if notif.kind == "subscription" {
                                            "notification-item subscription-notice"
                                        } else {
                                            "notification-item"
                                        };
                                        let read_class = if notif.is_read { "" } else { "unread" };
                                        view! {
                                            <div class=format!("{kind_class} {read_class}")>
                                                <p class="notification-title">
                                                    {if !notif.is_read { "● " } else { "" }}
                                                    {notif.title.clone()}
                                                </p>
                                                <p class="notification-body">{notif.body.clone()}</p>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>
                    }.into_any()
                },
                _ => view! { <></> }.into_any(),
            })}

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