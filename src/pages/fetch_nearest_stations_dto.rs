use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};

use crate::utils::base_url::BaseUrl;

#[derive(Deserialize)]
struct ApiErrorBody {
    message: String,
}

fn map_discount_generation_error(status: u16, message: Option<&str>) -> String {
    let raw = message.unwrap_or_default().trim();
    let normalized = raw.to_ascii_lowercase();

    if normalized.contains("daily discount code limit reached") {
        return "You’ve already generated a discount code for this station today. Please try again tomorrow.".to_string();
    }

    if normalized.contains("discount is not enabled") {
        return "Discounts are not available for this station right now.".to_string();
    }

    if normalized.contains("discount percentage is not configured") {
        return "Discount is temporarily unavailable. Please try again later.".to_string();
    }

    if normalized.contains("unable to determine caller ip") {
        return "We couldn’t verify your request right now. Please retry.".to_string();
    }

    if normalized.contains("not found") {
        return "Station details were not found. Please refresh and try again.".to_string();
    }

    match status {
        400 => "Unable to generate discount code. Please check your request and try again.".to_string(),
        401 | 403 => "You are not allowed to generate a discount code right now.".to_string(),
        404 => "Station details were not found. Please refresh and try again.".to_string(),
        429 => "Too many attempts. Please wait a bit and try again.".to_string(),
        500..=599 => "The server is busy right now. Please try again in a moment.".to_string(),
        _ if !raw.is_empty() => raw.to_string(),
        _ => "Couldn’t generate a discount code right now. Please try again.".to_string(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commodity {
    pub id: String,
    pub name: String,
    pub price: i32,
    pub station_id: String,
    pub is_available: bool,
    pub discount_enabled: Option<bool>,
    pub discount_percentage: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiscountCodeResponse {
    pub code: String,
    pub created_at: String,
    pub expires_at: String,
    pub discount_percentage: i32,
    pub original_price: i32,
    pub discounted_price: i32,
    pub is_expired: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub station_type: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
    pub distance: Option<f64>,
    pub commodities: Vec<Commodity>,
}

pub async fn fetch_closests(lat:f64, lon:f64, station_type:String) -> Result<Vec<Station>, String> {
    let BASE_URL = BaseUrl::get_base_url();
    let url = 
    format!("{BASE_URL}/api/v1/stations/closest?latitude={lat}&longitude={lon}&station_type={station_type}"); // Added "stations" to match typical API
    let mut last_error = "Unknown error".to_string();

    for attempt in 0..=1 {
        let request = Request::get(url.as_str()).send().await;

        match request {
            Ok(resp) => {
                if resp.ok() {
                    return resp
                        .json::<Vec<Station>>()
                        .await
                        .map_err(|e| format!("Parsing error: {}", e));
                }

                let status = resp.status();
                last_error = format!("Server error: {status}");

                // Retry once for transient availability issues (cold-start / gateway).
                if attempt == 0 && (status == 503 || status == 502 || status == 504) {
                    TimeoutFuture::new(700).await;
                    continue;
                }

                return Err(last_error);
            }
            Err(e) => {
                last_error = format!("Network error: {}", e);

                // Retry once for transient network startup issues.
                if attempt == 0 {
                    TimeoutFuture::new(700).await;
                    continue;
                }

                return Err(last_error);
            }
        }
    }

    Err(last_error)
}

pub async fn generate_discount_code(station_id: String) -> Result<DiscountCodeResponse, String> {
    let base_url = BaseUrl::get_base_url();
    let url = format!("{base_url}/api/v1/discounts/generate");
    let payload = serde_json::json!({ "station_id": station_id });

    let resp = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|_| "Couldn’t prepare your discount request. Please try again.".to_string())?
        .send()
        .await
        .map_err(|_| "Network issue while generating discount code. Check your connection and retry.".to_string())?;

    if resp.ok() {
        resp.json::<DiscountCodeResponse>()
            .await
            .map_err(|_| "Discount code was created, but we couldn’t read the response. Please try again.".to_string())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let parsed_message = serde_json::from_str::<ApiErrorBody>(&text)
            .ok()
            .map(|b| b.message)
            .or_else(|| if text.trim().is_empty() { None } else { Some(text) });

        Err(map_discount_generation_error(status, parsed_message.as_deref()))
    }
}