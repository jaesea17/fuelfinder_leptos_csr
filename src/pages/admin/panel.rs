use leptos::prelude::*;
use web_sys::js_sys::Date;
use wasm_bindgen::JsCast;

use crate::pages::admin::dto::{
    StationWithSubscription, clear_admin_password, fetch_admin_stations, fetch_discount_stats,
    get_admin_password, renew_station_subscription, set_admin_password, update_station_discount,
};

fn station_matches_filter(station: &StationWithSubscription, filter: &str) -> bool {
    match filter {
        "active" => station.subscription_status.as_deref() == Some("active"),
        "expired" => station.subscription_status.as_deref() != Some("active"),
        _ => true,
    }
}

fn renewed_subscription_end_date(days: i64) -> String {
    let date = Date::new_0();
    let next_day = (date.get_date() as u32).saturating_add(days.max(0) as u32);
    date.set_date(next_day);

    format!(
        "{:04}-{:02}-{:02}",
        date.get_full_year() as i32,
        date.get_month() + 1,
        date.get_date()
    )
}

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
    let stations = RwSignal::new(Vec::<StationWithSubscription>::new());
    let renewing_station: RwSignal<Option<StationWithSubscription>> = RwSignal::new(None);
    let renew_days = RwSignal::new("30".to_string());
    let renew_error = RwSignal::new(None::<String>);
    let discount_error = RwSignal::new(None::<String>);
    let expanded_station_id = RwSignal::new(None::<String>);
    let enabling_commodity_id = RwSignal::new(None::<String>);
    let enable_percentage_input = RwSignal::new("5".to_string());
    let enable_submit_commodity_id = RwSignal::new(None::<String>);
    let disable_submit_commodity_id = RwSignal::new(None::<String>);

    let stations_resource = LocalResource::new(move || {
        let filter = active_filter.get();
        async move {
            let pw = get_admin_password();
            fetch_admin_stations(pw, filter).await
        }
    });

    let discount_stats_resource = LocalResource::new(move || {
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
            renew_station_subscription(id.clone(), d, pw).await.map(|_| (id, d))
        }
    });

    let discount_action = Action::new_local(
        move |(commodity_id, enabled, percentage): &(String, bool, Option<i32>)| {
            let commodity_id = commodity_id.clone();
            let enabled = *enabled;
            let percentage = *percentage;

            async move {
                let pw = get_admin_password();
                update_station_discount(commodity_id.clone(), enabled, percentage, pw)
                    .await
                    .map(|_| (commodity_id, enabled, percentage))
            }
        },
    );

    Effect::new(move |_| {
        if let Some(Ok(fetched_stations)) = stations_resource.get() {
            stations.set(fetched_stations);
        }
    });

    Effect::new(move |_| {
        if let Some(result) = renew_action.value().get() {
            match result {
                Ok((station_id, days)) => {
                    let filter = active_filter.get();
                    let next_expiry = renewed_subscription_end_date(days);

                    stations.update(|rows| {
                        if let Some(station) = rows.iter_mut().find(|row| row.id == station_id) {
                            station.subscription_status = Some("active".to_string());
                            station.subscription_ends_at = Some(next_expiry.clone());
                        }

                        rows.retain(|station| station_matches_filter(station, &filter));
                    });

                    renewing_station.set(None);
                    renew_error.set(None);
                }
                Err(e) => renew_error.set(Some(e)),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = discount_action.value().get() {
            match result {
                Ok((commodity_id, enabled, percentage)) => {
                    stations.update(|rows| {
                        if let Some(station) = rows.iter_mut().find(|row| row.commodity_id.as_deref() == Some(commodity_id.as_str())) {
                            station.discount_enabled = Some(enabled);
                            station.discount_percentage = if enabled { percentage } else { None };
                        }
                    });

                    discount_error.set(None);
                    enable_submit_commodity_id.set(None);
                    disable_submit_commodity_id.set(None);
                    enabling_commodity_id.set(None);
                }
                Err(e) => {
                    enable_submit_commodity_id.set(None);
                    disable_submit_commodity_id.set(None);
                    discount_error.set(Some(e));
                }
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
                    Ok(_) if stations.get().is_empty() => view! {
                        <p class="empty-state">"No stations found for this filter."</p>
                    }.into_any(),

                    Ok(_) => {
                        let station_rows = stations.get();

                        view! {
                        <div class="admin-summary-strip">
                            <div class="admin-summary-card">
                                <span class="admin-summary-label">"Total Stations"</span>
                                <strong class="admin-summary-value">{station_rows.len().to_string()}</strong>
                            </div>
                            <div class="admin-summary-card">
                                <span class="admin-summary-label">"Active Subscriptions"</span>
                                <strong class="admin-summary-value">
                                    {station_rows
                                        .iter()
                                        .filter(|s| s.subscription_status.as_deref() == Some("active"))
                                        .count()
                                        .to_string()}
                                </strong>
                            </div>
                            <div class="admin-summary-card">
                                <span class="admin-summary-label">"Expired / No Active Plan"</span>
                                <strong class="admin-summary-value">
                                    {station_rows
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
                                        <th>"Action"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || station_rows.clone()
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
                                            let commodity_id_for_input = commodity_id.clone();
                                            let commodity_id_for_enable_disabled = commodity_id.clone();
                                            let commodity_id_for_enable_click = commodity_id.clone();
                                            let commodity_id_for_enable_label = commodity_id.clone();
                                            let commodity_id_for_disable_disabled = commodity_id.clone();
                                            let commodity_id_for_disable_click = commodity_id.clone();
                                            let commodity_id_for_disable_label = commodity_id.clone();
                                            let discount_enabled = station.discount_enabled.unwrap_or(false);
                                            let current_discount = station.discount_percentage.unwrap_or(5);
                                            let station_id_for_toggle = station.id.clone();
                                            let station_id_for_label = station.id.clone();
                                            let station_id_for_panel = station.id.clone();
                                            let expanded_for_toggle = expanded_station_id;
                                            let expanded_for_label = expanded_station_id;
                                            let expanded_for_panel = expanded_station_id;
                                            let station_email = station.email.clone();
                                            let station_type = station.station_type.clone();
                                            let station_created_count = station.discount_created_count;
                                            let station_redeemed_count = station.discount_redeemed_count;

                                            view! {
                                                <tr>
                                                    <td data-label="Name">{station.name.clone()}</td>
                                                    <td data-label="Address" class="station-address">{station.address.clone()}</td>
                                                    <td data-label="Action">
                                                        <div style="display: flex; flex-direction: column; gap: 8px; align-items: flex-start;">
                                                            <button
                                                                class="renew-button"
                                                                on:click=move |_| {
                                                                    if expanded_for_toggle.get().as_deref() == Some(station_id_for_toggle.as_str()) {
                                                                        expanded_for_toggle.set(None);
                                                                    } else {
                                                                        expanded_for_toggle.set(Some(station_id_for_toggle.clone()));
                                                                    }
                                                                }
                                                            >
                                                                {move || {
                                                                    if expanded_for_label.get().as_deref() == Some(station_id_for_label.as_str()) {
                                                                        "Hide info"
                                                                    } else {
                                                                        "More info"
                                                                    }
                                                                }}
                                                            </button>

                                                            <div
                                                                style="padding: 6px 0 2px; gap: 8px;"
                                                                style:display=move || {
                                                                    if expanded_for_panel.get().as_deref() == Some(station_id_for_panel.as_str()) {
                                                                        "grid"
                                                                    } else {
                                                                        "none"
                                                                    }
                                                                }
                                                            >
                                                                <div><strong>"Email: "</strong>{station_email.clone()}</div>
                                                                <div><strong>"Type: "</strong>{station_type.clone()}</div>
                                                                <div>
                                                                    <strong>"Status: "</strong>
                                                                    <span class=status_class>{status.clone()}</span>
                                                                </div>
                                                                <div><strong>"Expires: "</strong>{expires.clone()}</div>
                                                                <div>
                                                                    <strong>"Discount: "</strong>
                                                                    {if discount_enabled { "Enabled" } else { "Disabled" }}
                                                                </div>
                                                                <div>
                                                                    <strong>"Codes: "</strong>
                                                                    {format!("created {}, redeemed {}", station_created_count, station_redeemed_count)}
                                                                </div>

                                                                <div style="display:flex; gap:6px; flex-wrap:wrap; margin-top: 4px;">
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
                                                                        disabled=move || {
                                                                            discount_action.pending().get()
                                                                                || commodity_id_for_enable_disabled.is_none()
                                                                                || discount_enabled
                                                                        }
                                                                        on:click=move |_| {
                                                                            if let Some(cid) = commodity_id_for_enable_click.clone() {
                                                                                let selecting_this = enabling_commodity_id
                                                                                    .get()
                                                                                    .as_deref()
                                                                                    == Some(cid.as_str());

                                                                                if !selecting_this {
                                                                                    enabling_commodity_id.set(Some(cid));
                                                                                    enable_percentage_input.set(current_discount.to_string());
                                                                                    return;
                                                                                }

                                                                                let percentage = enable_percentage_input
                                                                                    .get()
                                                                                    .parse::<i32>()
                                                                                    .ok()
                                                                                    .filter(|v| (1..=10).contains(v));

                                                                                if percentage.is_some() {
                                                                                    enable_submit_commodity_id.set(Some(cid.clone()));
                                                                                    discount_action.dispatch((cid, true, percentage));
                                                                                } else {
                                                                                    discount_error.set(Some("Percentage must be an integer between 1 and 10".to_string()));
                                                                                }
                                                                            }
                                                                        }
                                                                    >
                                                                        {move || {
                                                                            if let Some(cid) = commodity_id_for_enable_label.clone() {
                                                                                if discount_action.pending().get() && enable_submit_commodity_id.get().as_deref() == Some(cid.as_str()) {
                                                                                    "Enabling..."
                                                                                } else {
                                                                                    "Enable"
                                                                                }
                                                                            } else {
                                                                                "Enable"
                                                                            }
                                                                        }}
                                                                    </button>

                                                                    <button
                                                                        class="cancel-button"
                                                                        disabled=move || {
                                                                            discount_action.pending().get()
                                                                                || commodity_id_for_disable_disabled.is_none()
                                                                                || !discount_enabled
                                                                        }
                                                                        on:click=move |_| {
                                                                            if let Some(cid) = commodity_id_for_disable_click.clone() {
                                                                                disable_submit_commodity_id.set(Some(cid.clone()));
                                                                                discount_action.dispatch((cid, false, None));
                                                                                enabling_commodity_id.set(None);
                                                                            }
                                                                        }
                                                                    >
                                                                        {move || {
                                                                            if let Some(cid) = commodity_id_for_disable_label.clone() {
                                                                                if discount_action.pending().get() && disable_submit_commodity_id.get().as_deref() == Some(cid.as_str()) {
                                                                                    "Disabling..."
                                                                                } else {
                                                                                    "Disable"
                                                                                }
                                                                            } else {
                                                                                "Disable"
                                                                            }
                                                                        }}
                                                                    </button>
                                                                </div>

                                                                {move || {
                                                                    if let Some(cid) = commodity_id_for_input.clone() {
                                                                        if enabling_commodity_id.get().as_deref() == Some(cid.as_str()) {
                                                                            return view! {
                                                                                <div style="display: flex; gap: 8px; align-items: center; margin-top: 6px;">
                                                                                    <label><strong>"Discount %"</strong></label>
                                                                                    <input
                                                                                        type="number"
                                                                                        min="1"
                                                                                        max="10"
                                                                                        class="price-input"
                                                                                        style="max-width: 110px;"
                                                                                        prop:value=move || enable_percentage_input.get()
                                                                                        on:input=move |ev| enable_percentage_input.set(event_target_value(&ev))
                                                                                    />
                                                                                    <small>"Click Enable again to confirm"</small>
                                                                                </div>
                                                                            }
                                                                                .into_any();
                                                                        }
                                                                    }

                                                                    view! { <span></span> }.into_any()
                                                                }}
                                                            </div>
                                                        </div>
                                                    </td>
                                                </tr>
                                            }
                                                .into_any()
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    }
                    .into_any()
                    },

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
