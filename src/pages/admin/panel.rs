use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::pages::admin::dto::{
    StationWithSubscription, clear_admin_password, fetch_admin_stations, fetch_discount_stats,
    get_admin_password, renew_station_subscription, set_admin_password, update_station_discount,
};

// ── Top-level component ──────────────────────────────────────────────────────

#[component]
pub fn AdminPanel() -> impl IntoView {
    let is_logged_in = RwSignal::new(!get_admin_password().is_empty());

    view! {
        <div class="admin-container">
            <Show
                when=move || is_logged_in.get()
                fallback=move || view! { <AdminLoginForm is_logged_in /> }
            >
                <AdminDashboardView is_logged_in />
            </Show>
        </div>
    }
}

// ── Login form ───────────────────────────────────────────────────────────────

#[component]
fn AdminLoginForm(is_logged_in: RwSignal<bool>) -> impl IntoView {
    let show_password = RwSignal::new(false);
    let error_msg = RwSignal::new(None::<String>);

    // Verify by attempting a real API call with the supplied password
    let login_action = Action::new_local(move |password: &String| {
        let password = password.clone();
        async move {
            fetch_admin_stations(password.clone(), "all".to_string())
                .await
                .map(|_| password)
        }
    });

    Effect::new(move |_| {
        if let Some(result) = login_action.value().get() {
            match result {
                Ok(pw) => {
                    set_admin_password(&pw);
                    error_msg.set(None);
                    is_logged_in.set(true);
                }
                Err(e) => error_msg.set(Some(e)),
            }
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let form: web_sys::HtmlFormElement =
            ev.target().unwrap().unchecked_into();
        let form_data = web_sys::FormData::new_with_form(&form).unwrap();
        let password = form_data.get("password").as_string().unwrap_or_default();
        if !password.is_empty() {
            login_action.dispatch(password);
        }
    };

    view! {
        <div class="signin-page">
            <div class="form-container admin-login-card">
                <span class="admin-eyebrow">"Secure access"</span>
                <h2>"Admin Login"</h2>
                <p class="admin-login-subtitle">
                    "Sign in to review station subscriptions, filter expired accounts, and renew access."
                </p>
                <form on:submit=on_submit>
                    <div class="form-group">
                        <label>"Admin Password"</label>
                        <div class="password-wrapper">
                            <input
                                type=move || if show_password.get() { "text" } else { "password" }
                                name="password"
                                style="width: 100%; padding-right: 40px;"
                            />
                            <button
                                type="button"
                                class="password-toggle"
                                on:click=move |_| show_password.update(|v| *v = !*v)
                            >
                                {move || if show_password.get() { "hide" } else { "show" }}
                            </button>
                        </div>
                    </div>
                    <button
                        type="submit"
                        class="submit-button"
                        disabled=move || login_action.pending().get()
                    >
                        {move || if login_action.pending().get() { "Verifying..." } else { "Login" }}
                    </button>
                </form>
                {move || error_msg.get().map(|e| view! {
                    <small class="error-message">{e}</small>
                })}
            </div>
        </div>
    }
}

// ── Admin dashboard ──────────────────────────────────────────────────────────

#[component]
fn AdminDashboardView(is_logged_in: RwSignal<bool>) -> impl IntoView {
    let active_filter = RwSignal::new("all".to_string());
    let refresh_trigger = RwSignal::new(0u32);
    let renewing_station: RwSignal<Option<StationWithSubscription>> = RwSignal::new(None);
    let renew_days = RwSignal::new("30".to_string());
    let renew_error = RwSignal::new(None::<String>);
    let discount_error = RwSignal::new(None::<String>);
    let discount_percentage = RwSignal::new("5".to_string());

    // Reactive: re-fetches whenever active_filter or refresh_trigger changes
    let stations_resource = LocalResource::new(move || {
        let filter = active_filter.get();
        let _r = refresh_trigger.get();
        async move {
            let pw = get_admin_password();
            fetch_admin_stations(pw, filter).await
        }
    });

    let discount_stats_resource = LocalResource::new(move || {
        let _r = refresh_trigger.get();
        async move {
            let pw = get_admin_password();
            fetch_discount_stats(pw).await
        }
    });

    let renew_action = Action::new_local(move |(station_id, days): &(String, i64)| {
        let id = station_id.clone();
        let d = *days;
        async move {
            let pw = get_admin_password();
            renew_station_subscription(id, d, pw).await
        }
    });

    let discount_action = Action::new_local(
        move |(commodity_id, enabled, percentage): &(String, bool, Option<i32>)| {
            let commodity_id = commodity_id.clone();
            let enabled = *enabled;
            let percentage = *percentage;

            async move {
                let pw = get_admin_password();
                update_station_discount(commodity_id, enabled, percentage, pw).await
            }
        },
    );

    Effect::new(move |_| {
        if let Some(result) = renew_action.value().get() {
            match result {
                Ok(_) => {
                    renewing_station.set(None);
                    renew_error.set(None);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => renew_error.set(Some(e)),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = discount_action.value().get() {
            match result {
                Ok(_) => {
                    discount_error.set(None);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => discount_error.set(Some(e)),
            }
        }
    });

    let logout = move |_| {
        clear_admin_password();
        is_logged_in.set(false);
    };

    view! {
        <div class="admin-dashboard">

            // ── Header ───────────────────────────────────────────────────────
            <div class="admin-header">
                <div class="admin-header-copy">
                    <span class="admin-eyebrow">"FuelFinder control panel"</span>
                    <h1>"Admin Dashboard"</h1>
                    <p class="admin-subtitle">
                        "Monitor all stations, check subscription status, and renew accounts before service interruptions."
                    </p>
                </div>
                <button class="logout-button" on:click=logout>"Logout"</button>
            </div>

            // ── Filter tabs ──────────────────────────────────────────────────
            <div class="filter-tabs">
                <button
                    class=move || format!(
                        "filter-tab {}",
                        if active_filter.get() == "all" { "active-tab" } else { "" }
                    )
                    on:click=move |_| active_filter.set("all".to_string())
                >
                    "All Stations"
                </button>
                <button
                    class=move || format!(
                        "filter-tab {}",
                        if active_filter.get() == "active" { "active-tab" } else { "" }
                    )
                    on:click=move |_| active_filter.set("active".to_string())
                >
                    "Active"
                </button>
                <button
                    class=move || format!(
                        "filter-tab {}",
                        if active_filter.get() == "expired" { "active-tab" } else { "" }
                    )
                    on:click=move |_| active_filter.set("expired".to_string())
                >
                    "Expired"
                </button>
            </div>

            // ── Station table ────────────────────────────────────────────────
            <Suspense fallback=move || view! { <p class="loading">"Loading stations..."</p> }>
                {move || stations_resource.get().map(|res| match res {
                    Ok(stations) if stations.is_empty() => view! {
                        <p class="empty-state">"No stations found for this filter."</p>
                    }.into_any(),

                    Ok(stations) => view! {
                        <div class="admin-summary-strip">
                            <div class="admin-summary-card">
                                <span class="admin-summary-label">"Total Stations"</span>
                                <strong class="admin-summary-value">{stations.len().to_string()}</strong>
                            </div>
                            <div class="admin-summary-card">
                                <span class="admin-summary-label">"Active Subscriptions"</span>
                                <strong class="admin-summary-value">
                                    {stations
                                        .iter()
                                        .filter(|s| s.subscription_status.as_deref() == Some("active"))
                                        .count()
                                        .to_string()}
                                </strong>
                            </div>
                            <div class="admin-summary-card">
                                <span class="admin-summary-label">"Expired / No Active Plan"</span>
                                <strong class="admin-summary-value">
                                    {stations
                                        .iter()
                                        .filter(|s| s.subscription_status.as_deref() != Some("active"))
                                        .count()
                                        .to_string()}
                                </strong>
                            </div>
                            {move || discount_stats_resource.get().and_then(|res| res.ok()).map(|stats| view! {
                                <>
                                    <div class="admin-summary-card">
                                        <span class="admin-summary-label">"Discount Codes Created"</span>
                                        <strong class="admin-summary-value">{stats.created_codes.to_string()}</strong>
                                    </div>
                                    <div class="admin-summary-card">
                                        <span class="admin-summary-label">"Discount Codes Redeemed"</span>
                                        <strong class="admin-summary-value">{stats.redeemed_codes.to_string()}</strong>
                                    </div>
                                </>
                            })}
                        </div>

                        {move || discount_error.get().map(|e| view! {
                            <p class="error-message">{e}</p>
                        })}

                        <div class="stations-table-wrapper">
                            <table class="stations-table">
                                <thead>
                                    <tr>
                                        <th>"Name"</th>
                                        <th>"Address"</th>
                                        <th>"Email"</th>
                                        <th>"Type"</th>
                                        <th>"Status"</th>
                                        <th>"Expires"</th>
                                        <th>"Discount"</th>
                                        <th>"Discount %"</th>
                                        <th>"Action"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || stations.clone()
                                        key=|s| s.id.clone()
                                        children=move |station| {
                                            let station_for_renew = station.clone();
                                            let status = station
                                                .subscription_status
                                                .clone()
                                                .unwrap_or_else(|| "none".to_string());
                                            let expires = station
                                                .subscription_ends_at
                                                .clone()
                                                .map(|d| d.chars().take(10).collect::<String>())
                                                .unwrap_or_else(|| "N/A".to_string());

                                            let status_class = match status.as_str() {
                                                "active"   => "status-badge status-active",
                                                "expired"  => "status-badge status-expired",
                                                _          => "status-badge status-none",
                                            };

                                            let commodity_id = station.commodity_id.clone();
                                            let commodity_id_for_enable_disabled = commodity_id.clone();
                                            let commodity_id_for_enable_click = commodity_id.clone();
                                            let commodity_id_for_disable_disabled = commodity_id.clone();
                                            let commodity_id_for_disable_click = commodity_id.clone();
                                            let discount_enabled = station.discount_enabled.unwrap_or(false);
                                            let current_discount = station.discount_percentage.unwrap_or(5);

                                            view! {
                                                <tr>
                                                    <td data-label="Name">{station.name.clone()}</td>
                                                    <td data-label="Address" class="station-address">{station.address.clone()}</td>
                                                    <td data-label="Email">{station.email.clone()}</td>
                                                    <td data-label="Type">{station.station_type.clone()}</td>
                                                    <td data-label="Status">
                                                        <span class=status_class>{status}</span>
                                                    </td>
                                                    <td data-label="Expires">{expires}</td>
                                                    <td data-label="Discount">
                                                        {if discount_enabled { "Enabled" } else { "Disabled" }}
                                                    </td>
                                                    <td data-label="Discount %">
                                                        <input
                                                            type="number"
                                                            min="1"
                                                            max="10"
                                                            class="price-input"
                                                            style="max-width: 90px;"
                                                            prop:value=current_discount.to_string()
                                                            on:input=move |ev| discount_percentage.set(event_target_value(&ev))
                                                        />
                                                    </td>
                                                    <td data-label="Action">
                                                        <div style="display:flex; gap:6px; flex-wrap:wrap;">
                                                            <button
                                                                class="renew-button"
                                                                on:click=move |_| {
                                                                    renew_days.set("30".to_string());
                                                                    renew_error.set(None);
                                                                    renewing_station.set(Some(station_for_renew.clone()));
                                                                }
                                                            >
                                                                "Renew"
                                                            </button>

                                                            <button
                                                                class="save-button"
                                                                disabled=move || discount_action.pending().get() || commodity_id_for_enable_disabled.is_none()
                                                                on:click=move |_| {
                                                                    if let Some(cid) = commodity_id_for_enable_click.clone() {
                                                                        let percentage = discount_percentage
                                                                            .get()
                                                                            .parse::<i32>()
                                                                            .ok()
                                                                            .filter(|v| (1..=10).contains(v));
                                                                        discount_action.dispatch((cid, true, percentage));
                                                                    }
                                                                }
                                                            >
                                                                "Enable"
                                                            </button>

                                                            <button
                                                                class="cancel-button"
                                                                disabled=move || discount_action.pending().get() || commodity_id_for_disable_disabled.is_none()
                                                                on:click=move |_| {
                                                                    if let Some(cid) = commodity_id_for_disable_click.clone() {
                                                                        discount_action.dispatch((cid, false, None));
                                                                    }
                                                                }
                                                            >
                                                                "Disable"
                                                            </button>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    }.into_any(),

                    Err(e) => view! {
                        <p class="error-message">{e}</p>
                    }.into_any(),
                })}
            </Suspense>

            // ── Renewal modal ────────────────────────────────────────────────
            {move || renewing_station.get().map(|station| {
                let station_id = station.id.clone();
                let station_name = station.name.clone();

                view! {
                    <div class="modal-overlay">
                        <div class="modal admin-modal">
                            <h2>"Renew Subscription"</h2>
                            <p class="admin-modal-station">"Station: " <strong>{station_name}</strong></p>

                            <div class="form-group">
                                <label>"Duration (days)"</label>
                                <input
                                    type="number"
                                    class="price-input"
                                    prop:value=move || renew_days.get()
                                    on:input=move |ev| renew_days.set(event_target_value(&ev))
                                    min="1"
                                />
                            </div>

                            {move || renew_error.get().map(|e| view! {
                                <small class="error-message">{e}</small>
                            })}

                            <div class="modal-actions">
                                <button
                                    class="save-button"
                                    disabled=move || renew_action.pending().get()
                                    on:click=move |_| {
                                        if let Ok(days) = renew_days.get().parse::<i64>() {
                                            renew_action.dispatch((station_id.clone(), days));
                                        }
                                    }
                                >
                                    {move || if renew_action.pending().get() { "Renewing..." } else { "Confirm Renewal" }}
                                </button>
                                <button
                                    class="cancel-button"
                                    on:click=move |_| renewing_station.set(None)
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
