use gloo_net::http::Request;
use leptos::prelude::window;
use serde::{Deserialize, Serialize};

use crate::utils::base_url::BaseUrl;

fn map_admin_request_error(error: String) -> String {
    let normalized = error.to_lowercase();

    if normalized.contains("failed to fetch") || normalized.contains("networkerror") {
        "Unable to reach the admin API. This is usually a network or CORS issue. Confirm the server is running, the API base URL is correct, and the backend CORS config allows this origin and the X-Admin-Password header.".to_string()
    } else {
        error
    }
}

// ── Types ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationWithSubscription {
    pub id: String,
    pub name: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub station_type: String,
    pub commodity_id: Option<String>,
    pub discount_enabled: Option<bool>,
    pub discount_percentage: Option<i32>,
    pub discount_created_count: i64,
    pub discount_redeemed_count: i64,
    pub subscription_status: Option<String>,
    pub subscription_ends_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscountStats {
    pub created_codes: i64,
    pub redeemed_codes: i64,
}

// ── localStorage helpers ─────────────────────────────────────────────────────

pub fn get_admin_password() -> String {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item("adminPassword").ok().flatten())
        .unwrap_or_default()
}

pub fn set_admin_password(password: &str) {
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.set_item("adminPassword", password);
    }
}

pub fn clear_admin_password() {
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.remove_item("adminPassword");
    }
}

// ── API functions ────────────────────────────────────────────────────────────

pub async fn fetch_admin_stations(
    password: String,
    filter: String,
) -> Result<Vec<StationWithSubscription>, String> {
    if password.trim().is_empty() {
        return Err("Admin password is required".to_string());
    }

    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/admin/stations?filter={filter}");

    let resp = Request::get(&url)
        .header("X-Admin-Password", &password)
        .send()
        .await
        .map_err(|e| map_admin_request_error(e.to_string()))?;

    if resp.ok() {
        resp.json::<Vec<StationWithSubscription>>()
            .await
            .map_err(|e| map_admin_request_error(e.to_string()))
    } else if resp.status() == 401 || resp.status() == 403 {
        Err("Invalid admin password".to_string())
    } else {
        Err(format!("Server error: {}", resp.status()))
    }
}

pub async fn renew_station_subscription(
    station_id: String,
    days: i64,
    admin_password: String,
) -> Result<(), String> {
    if admin_password.trim().is_empty() {
        return Err("Admin password is required".to_string());
    }

    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/auth/subscriptions/renew");

    let payload = serde_json::json!({
        "station_id": station_id,
        "days": days,
        "super_password": admin_password,
    });

    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| map_admin_request_error(e.to_string()))?;

    if resp.ok() {
        Ok(())
    } else {
        let err_text = resp.text().await.unwrap_or_default();
        Err(format!("Server error: {err_text}"))
    }
}

pub async fn update_station_discount(
    commodity_id: String,
    enabled: bool,
    percentage: Option<i32>,
    admin_password: String,
) -> Result<(), String> {
    if admin_password.trim().is_empty() {
        return Err("Admin password is required".to_string());
    }

    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/admin/discounts/{commodity_id}");

    let payload = serde_json::json!({
        "commodity_id": commodity_id,
        "enabled": enabled,
        "percentage": percentage,
    });

    let resp = Request::patch(&url)
        .header("X-Admin-Password", &admin_password)
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| map_admin_request_error(e.to_string()))?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Server error: {}", resp.status()))
    }
}

pub async fn fetch_discount_stats(admin_password: String) -> Result<DiscountStats, String> {
    if admin_password.trim().is_empty() {
        return Err("Admin password is required".to_string());
    }

    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/admin/discounts/stats");

    let resp = Request::get(&url)
        .header("X-Admin-Password", &admin_password)
        .send()
        .await
        .map_err(|e| map_admin_request_error(e.to_string()))?;

    if resp.ok() {
        resp.json::<DiscountStats>()
            .await
            .map_err(|e| map_admin_request_error(e.to_string()))
    } else {
        Err(format!("Server error: {}", resp.status()))
    }
}
