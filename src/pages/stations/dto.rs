use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use leptos::prelude::*;

use crate::{pages::fetch_nearest_stations_dto::Station, utils::base_url};
use crate::utils::base_url::BaseUrl;


#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RegisterFormData {
    pub name: String,
    pub address: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub code: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginResponse {
    pub access_token: String,
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
                "latitude": lat,
                "longitude": lon, 
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
                // If 4xx or 5xx status code
                Err(format!("Server error: {}", resp.status()))
            }
        }
        // If network failed entirely
        Err(e) => Err(format!("Network error: {}", e)),
    }
}