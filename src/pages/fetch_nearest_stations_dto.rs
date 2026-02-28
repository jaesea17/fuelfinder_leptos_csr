use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::utils::base_url::BaseUrl;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commodity {
    pub id: String,
    pub name: String,
    pub price: i32,
    pub station_id: String,
    pub is_available: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub latitude: f64,
    pub longitude: f64,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
    pub distance: Option<f64>,
    pub commodities: Vec<Commodity>,
}

pub async fn fetch_closests(lat:f64, lon:f64) -> Result<Vec<Station>, String> {
    let BASE_URL = BaseUrl::get_base_url();
    let url = format!("{BASE_URL}/api/v1/stations/closest?latitude={lat}&longitude={lon}"); // Added "stations" to match typical API
    let request = Request::get(url.as_str()).send().await;

    match request {
        Ok(resp) => {
            if resp.ok() {
                // If 200-299 status code
                resp.json::<Vec<Station>>().await.map_err(|e| format!("Parsing error: {}", e))
            } else {
                // If 4xx or 5xx status code
                Err(format!("Server error: {}", resp.status()))
            }
        }
        // If network failed entirely
        Err(e) => Err(format!("Network error: {}", e)),
    }
}