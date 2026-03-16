use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::{pages::fetch_nearest_stations_dto::Station, utils::base_url};
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

pub async fn register_station(payload: RegisterFormData, lat: f64, lon:f64) -> Result<Station, String> {
    let BASE_URL = BaseUrl::get_base_url();
    let url = format!("{BASE_URL}/api/v1/auth/signup"); // Added "stations" to match typical API
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
                Err(format!("Server error: {}", resp.status()))
            }
        }
        // If network failed entirely
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn login_station(payload: LoginFormData) -> Result<LoginResponse, String> {
    let BASE_URL = BaseUrl::get_base_url();
    let url = format!("{BASE_URL}/api/v1/auth/signin"); // Added "stations" to match typical API
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
                let msg = resp
                    .json::<ApiErrorBody>()
                    .await
                    .map(|b| b.message)
                    .unwrap_or_else(|_| format!("Server error: {}", resp.status()));
                Err(msg)
            }
        }
        // If network failed entirely
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn generate_reg_code(code: String, super_password: String) -> Result<String, String> {
    let BASE_URL = BaseUrl::get_base_url();
    let url = format!("{BASE_URL}/api/v1/auth/reg-code");
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
                Err(format!("Server error: {}", resp.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn fetch_station_notifications(token: String) -> Result<Vec<DashboardNotification>, String> {
    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/stations/dashboard/notifications");
    let resp = Request::get(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<Vec<DashboardNotification>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Server error: {}", resp.status()))
    }
}

pub async fn mark_station_notification_read(notification_id: String, token: String) -> Result<(), String> {
    let base_url = BaseUrl::get_base_url();
    let url = format!(
        "{base_url}/api/v1/stations/dashboard/notifications/{notification_id}/read"
    );

    let resp = Request::patch(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        Ok(())
    } else {
        Err(format!("Server error: {}", resp.status()))
    }
}

pub async fn redeem_discount_code(
    token: String,
    code: String,
) -> Result<RedeemDiscountCodeResponse, String> {
    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/discounts/redeem");
    let payload = serde_json::json!({ "code": code });

    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.ok() {
        resp.json::<RedeemDiscountCodeResponse>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Server error: {}", resp.status()))
    }
}

pub async fn fetch_station_discount_stats(token: String) -> Result<StationDiscountStats, String> {
    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/discounts/station/stats");

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
        Err(format!("Server error: {}", resp.status()))
    }
}