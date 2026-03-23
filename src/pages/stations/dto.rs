use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::pages::fetch_nearest_stations_dto::Station;
use crate::utils::base_url::BaseUrl;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DashboardNotification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub is_read: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RedeemDiscountCodeResponse {
    pub message: String,
    pub code: Option<String>,
    pub created_at: Option<String>,
    pub expires_at: Option<String>,
    pub discount_percentage: Option<i32>,
    pub discounted_price: Option<i32>,
    pub is_expired: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StationDiscountStats {
    pub redeemed_codes: i64,
}


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RegisterFormData {
    pub name: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub code: String,
    pub station_type: String, // "petrol" or "cooking gas"
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
    pub station_type: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub access_token: String,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    message: String,
}

fn map_discount_redeem_error(status: u16, message: Option<&str>) -> String {
    let raw = message.unwrap_or_default().trim();
    let normalized = raw.to_ascii_lowercase();

    if normalized.contains("discount code not found") {
        return "Invalid code".to_string();
    }

    if normalized.contains("does not belong to your station") {
        return "This discount code belongs to another station and cannot be redeemed here.".to_string();
    }

    if normalized.contains("code is expired") {
        return "This discount code has expired and can no longer be redeemed.".to_string();
    }

    if normalized.contains("already redeemed") {
        return "This discount code has already been redeemed.".to_string();
    }

    match status {
        400 | 404 => "This code is invalid".to_string(),
        401 | 403 => "Your session may have expired. Please sign in again.".to_string(),
        409 => "This discount code has already been used.".to_string(),
        500..=599 => "Couldn’t redeem the discount right now. Please try again shortly.".to_string(),
        _ if !raw.is_empty() => raw.to_string(),
        _ => "Unable to redeem discount code right now. Please try again.".to_string(),
    }
}

pub fn map_status_server_error(status: u16) -> String {
    format!("Server error: {status}")
}

pub fn parse_api_error_message(status: u16, body_text: &str) -> String {
    serde_json::from_str::<ApiErrorBody>(body_text)
        .ok()
        .map(|b| b.message)
        .unwrap_or_else(|| map_status_server_error(status))
}

pub fn signup_url(base_url: &str) -> String {
    format!("{base_url}/api/v1/auth/signup")
}

pub fn signin_url(base_url: &str) -> String {
    format!("{base_url}/api/v1/auth/signin")
}

pub fn reg_code_url(base_url: &str) -> String {
    format!("{base_url}/api/v1/auth/reg-code")
}

pub fn station_notifications_url(base_url: &str) -> String {
    format!("{base_url}/api/v1/stations/dashboard/notifications")
}

pub fn station_notification_read_url(base_url: &str, notification_id: &str) -> String {
    format!("{base_url}/api/v1/stations/dashboard/notifications/{notification_id}/read")
}

pub fn discounts_redeem_url(base_url: &str) -> String {
    format!("{base_url}/api/v1/discounts/redeem")
}

pub fn station_discount_stats_url(base_url: &str) -> String {
    format!("{base_url}/api/v1/discounts/station/stats")
}

pub fn map_discount_redeem_error_message(status: u16, message: Option<&str>) -> String {
    map_discount_redeem_error(status, message)
}

pub async fn register_station(payload: RegisterFormData, lat: f64, lon:f64) -> Result<Station, String> {
    let base_url = BaseUrl::get_base_url();
    let url = signup_url(&base_url);
    let payload = serde_json::json!({
                "name": payload.name,
                "address": payload.address,
                "email": payload.email,
                "phone": payload.phone,
                "password": payload.password,
                "code": payload.code,
                "station_type": payload.station_type,
                "latitude": lat,
                "longitude": lon
            });
    let request = Request::post(url.as_str())
        .header("Content-Type", "application/json")
        .json(&payload) // This serializes the JSON and sends it
        .map_err(|e| e.to_string())?
        .send()
        .await;

    match request {
        Ok(resp) => {
            if resp.ok() {
                // If 200-299 status code
                resp.json::<Station>().await.map_err(|e| format!("Parsing error: {}", e))
            } else {
                // If 4xx or 5xx status code
                Err(map_status_server_error(resp.status()))
            }
        }
        // If network failed entirely
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn login_station(payload: LoginFormData) -> Result<LoginResponse, String> {
    let base_url = BaseUrl::get_base_url();
    let url = signin_url(&base_url);
    let request = Request::post(url.as_str())
        .header("Content-Type", "application/json")
        .json(&payload) // This serializes the JSON and sends it
        .map_err(|e| e.to_string())?
        .send()
        .await;

    match request {
        Ok(resp) => {
            if resp.ok() {
                // If 200-299 status code
                let response:LoginResponse = resp.json().await.map_err(|e| format!("Error while parsing, {}",e.to_string()))?;
                Ok(response)
            } else {
                // If 4xx or 5xx status code — parse the JSON body for the real message
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                let msg = parse_api_error_message(status, &text);
                Err(msg)
            }
        }
        // If network failed entirely
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn generate_reg_code(code: String, super_password: String) -> Result<String, String> {
    let base_url = BaseUrl::get_base_url();
    let url = reg_code_url(&base_url);
    let payload = serde_json::json!({
        "code": code,
        "super_password": super_password
    });
    
    let request = Request::post(url.as_str())
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await;

    match request {
        Ok(resp) => {
            if resp.ok() {
                // Return the code back on success
                Ok(code)
            } else {
                Err(map_status_server_error(resp.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn fetch_station_notifications(token: String) -> Result<Vec<DashboardNotification>, String> {
    let base_url = BaseUrl::get_base_url();
    let url = station_notifications_url(&base_url);
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<Vec<DashboardNotification>>().await.map_err(|e| e.to_string())
    } else {
        Err(map_status_server_error(resp.status()))
    }
}

pub async fn mark_station_notification_read(notification_id: String, token: String) -> Result<(), String> {
    let base_url = BaseUrl::get_base_url();
    let url = station_notification_read_url(&base_url, &notification_id);

    let resp = Request::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(map_status_server_error(resp.status()))
    }
}

pub async fn redeem_discount_code(
    token: String,
    code: String,
) -> Result<RedeemDiscountCodeResponse, String> {
    let base_url = BaseUrl::get_base_url();
    let url = discounts_redeem_url(&base_url);
    let payload = serde_json::json!({ "code": code });

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|_| "Couldn’t prepare discount redemption request. Please try again.".to_string())?
        .send()
        .await
        .map_err(|_| "Network issue while redeeming code. Check your connection and retry.".to_string())?;

    if resp.ok() {
        resp.json::<RedeemDiscountCodeResponse>()
            .await
            .map_err(|_| "Redemption completed, but we couldn’t read the response. Please refresh and try again.".to_string())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let parsed_message = serde_json::from_str::<ApiErrorBody>(&text)
            .ok()
            .map(|b| b.message)
            .or_else(|| if text.trim().is_empty() { None } else { Some(text) });

        Err(map_discount_redeem_error_message(status, parsed_message.as_deref()))
    }
}

pub async fn fetch_station_discount_stats(token: String) -> Result<StationDiscountStats, String> {
    let base_url = BaseUrl::get_base_url();
    let url = station_discount_stats_url(&base_url);

    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<StationDiscountStats>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(map_status_server_error(resp.status()))
    }
}