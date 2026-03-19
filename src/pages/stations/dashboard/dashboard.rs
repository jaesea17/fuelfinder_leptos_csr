use gloo_net::http::Request;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::collections::HashSet;

use crate::pages::fetch_nearest_stations_dto::Station;
use crate::pages::stations::dashboard::commodity_card::CommodityCard;
use crate::pages::stations::dashboard::utils::get_token;
use crate::pages::stations::dto::{
    fetch_station_discount_stats, fetch_station_notifications, mark_station_notification_read,
    redeem_discount_code,
};
use crate::utils::base_url::BaseUrl;

fn format_dashboard_date(value: &str) -> String {
    let date_only = value.split('T').next().unwrap_or(value);
    let mut parts = date_only.split('-');

    match (parts.next(), parts.next(), parts.next()) {
        (Some(year), Some(month), Some(day)) => format!("{}-{}-{}", day, month, year),
        _ => value.to_string(),
    }
}

#[component]
fn RedeemSection() -> impl IntoView {
    let discount_stats_resource = LocalResource::new(|| async move {
        let token = get_token();
        fetch_station_discount_stats(token).await
    });

    let redeemed_codes = RwSignal::new(None::<i64>);
    let redeem_code_input = RwSignal::new(String::new());
    let show_redeem_feedback = RwSignal::new(false);

    let redeem_action = Action::new_local(move |code: &String| {
        let code = code.clone();
        async move {
            let token = get_token();
            redeem_discount_code(token, code).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(stats)) = discount_stats_resource.get() {
            redeemed_codes.set(Some(stats.redeemed_codes));
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = redeem_action.value().get() {
            show_redeem_feedback.set(true);
            redeem_code_input.set(String::new());
            redeemed_codes.update(|count| {
                if let Some(value) = count.as_mut() {
                    *value += 1;
                }
            });
        }
    });

    view! {
        <div class="redeem-section">
            <div class="redeem-header">
                <span class="bell-icon">"🎟️"</span>
                <strong>"Discount Code Redemption"</strong>
            </div>

            {move || redeemed_codes.get().map(|count| view! {
                <p class="redeem-stats">{format!("Total redeemed codes: {}", count)}</p>
            })}

            <div class="redeem-form">
                <input
                    class="price-input redeem-input"
                    placeholder="Enter customer discount code"
                    prop:value=move || redeem_code_input.get()
                    on:input=move |ev| redeem_code_input.set(event_target_value(&ev).to_ascii_uppercase())
                />
                <button
                    class="save-button redeem-button"
                    disabled=move || redeem_action.pending().get() || redeem_code_input.get().trim().is_empty()
                    on:click=move |_| {
                        redeem_action.dispatch(redeem_code_input.get());
                    }
                >
                    {move || if redeem_action.pending().get() { "Redeeming..." } else { "Redeem Code" }}
                </button>
            </div>

            {move || show_redeem_feedback.get().then_some(()).and_then(|_| redeem_action.value().get()).map(|res| match res {
                Ok(resp) => {
                    let created = resp.created_at.clone();
                    let expires = resp.expires_at.clone();
                    let percentage = resp.discount_percentage;
                    let discounted = resp.discounted_price;

                    view! {
                        <div class="redeem-feedback notification-item subscription-notice">
                            <div class="redeem-feedback-header">
                                <p class="notification-title">{resp.message}</p>
                                <button
                                    class="redeem-close-btn"
                                    on:click=move |_| show_redeem_feedback.set(false)
                                >
                                    "✕"
                                </button>
                            </div>
                            {created.map(|v| view! { <p class="notification-body redeem-meta">{format!("Created: {}", format_dashboard_date(&v))}</p> })}
                            {expires.map(|v| view! { <p class="notification-body redeem-meta">{format!("Expires: {}", format_dashboard_date(&v))}</p> })}
                            {percentage.map(|v| view! { <p class="notification-body redeem-meta">{format!("Discount: {}%", v)}</p> })}
                            {discounted.map(|v| view! { <p class="notification-body redeem-meta">{format!("Sell at: ₦{}", v)}</p> })}
                        </div>
                    }.into_any()
                },
                Err(err) => view! { <small class="error-message">{err}</small> }.into_any(),
            })}
        </div>
    }
}

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

    let hidden_notification_ids = RwSignal::new(HashSet::<String>::new());

    let mark_notification_action = Action::new_local(move |notification_id: &String| {
        let id = notification_id.clone();
        async move {
            let token = get_token();
            mark_station_notification_read(id.clone(), token)
                .await
                .map(|_| id)
        }
    });

    Effect::new(move |_| {
        if let Some(result) = mark_notification_action.value().get() {
            if let Ok(notification_id) = result {
                hidden_notification_ids.update(|ids| {
                    ids.insert(notification_id);
                });
                notifications_resource.refetch();
            }
        }
    });

    // Auto-mark subscription renewal notifications as read
    Effect::new(move |_| {
        if let Some(Ok(notifs)) = notifications_resource.get() {
            for notif in notifs {
                if !notif.is_read && notif.kind == "subscription" {
                    mark_notification_action.dispatch(notif.id);
                }
            }
        }
    });

    view! {
        <div class="station-dashboard-page">
        <div class="station-dashboard">
            // Notification banner — renders as soon as ready, never blocks station content
            {move || notifications_resource.get().map(|res| match res {
                Ok(notifs) if !notifs.is_empty() => {
                    let hidden_ids = hidden_notification_ids.get();
                    let unread_notifs: Vec<_> = notifs
                        .into_iter()
                        .filter(|n| !n.is_read && !hidden_ids.contains(&n.id))
                        .collect();
                    let unread_count = unread_notifs.len();

                    if unread_notifs.is_empty() {
                        return view! { <></> }.into_any();
                    }

                    view! {
                        <div class="notifications-panel">
                            <div class="notifications-header">
                                <span class="bell-icon">"🔔"</span>
                                <strong>
                                    {format!("Notifications  ({} unread)", unread_count)}
                                </strong>
                            </div>
                            <div class="notifications-list">
                                <For
                                    each=move || unread_notifs.clone()
                                    key=|n| n.id.clone()
                                    children=move |notif| {
                                        let kind_class = if notif.kind == "subscription" {
                                            "notification-item subscription-notice"
                                        } else {
                                            "notification-item"
                                        };
                                        let read_class = if notif.is_read { "" } else { "unread" };
                                        let notification_id = notif.id.clone();
                                        view! {
                                            <div
                                                class=format!("{kind_class} {read_class}")
                                                on:click=move |_| {
                                                    mark_notification_action.dispatch(notification_id.clone());
                                                }
                                            >
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
                    Ok(data) => {
                        let discount_enabled_for_station = data
                            .commodities
                            .iter()
                            .any(|commodity| commodity.discount_enabled.unwrap_or(false));

                        view! {
                            <h1>{data.name}</h1>
                            <div class="commodities-grid">
                                <For
                                    each=move || data.commodities.clone()
                                    key=|c| c.id.clone()
                                    children=move |commodity| {
                                        let st = data.station_type.clone().unwrap_or_default();
                                        view! {
                                            <CommodityCard
                                                commodity=commodity
                                                station_type=st
                                            />
                                        }
                                    }
                                />
                            </div>

                            <Show
                                when=move || discount_enabled_for_station
                                fallback=move || view! { <></> }
                            >
                                <RedeemSection />
                            </Show>
                        }
                        .into_any()
                    },
                    Err(_) => {
                        navigate("/signin", Default::default());
                        view! { <p>"Unauthorized - Redirecting..."</p> }.into_any()
                    }
                })}
            </Suspense>
        </div>
        </div>
    }
}