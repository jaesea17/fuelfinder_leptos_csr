mod common;

use fuelfinder_client::{
    pages::{
        fetch_nearest_stations_dto::{
            closest_stations_url,
            discount_generation_url,
            map_discount_generation_error_message,
        },
        stations::dto::{
            discounts_redeem_url,
            map_status_server_error,
            map_discount_redeem_error_message,
            parse_api_error_message,
            reg_code_url,
            signin_url,
            signup_url,
            station_discount_stats_url,
            station_notification_read_url,
            station_notifications_url,
        },
    },
};

use common::{BASE_URL, NOTIFICATION_ID};

#[test]
fn station_endpoint_urls_match_router() {
    assert_eq!(signup_url(BASE_URL), "https://api.example.com/api/v1/auth/signup");
    assert_eq!(signin_url(BASE_URL), "https://api.example.com/api/v1/auth/signin");
    assert_eq!(reg_code_url(BASE_URL), "https://api.example.com/api/v1/auth/reg-code");
    assert_eq!(
        station_notifications_url(BASE_URL),
        "https://api.example.com/api/v1/stations/dashboard/notifications"
    );
    assert_eq!(
        station_notification_read_url(BASE_URL, NOTIFICATION_ID),
        format!(
            "https://api.example.com/api/v1/stations/dashboard/notifications/{NOTIFICATION_ID}/read"
        )
    );
    assert_eq!(
        discounts_redeem_url(BASE_URL),
        "https://api.example.com/api/v1/discounts/redeem"
    );
    assert_eq!(
        station_discount_stats_url(BASE_URL),
        "https://api.example.com/api/v1/discounts/station/stats"
    );
}

#[test]
fn nearest_station_endpoint_urls_match_router() {
    assert_eq!(
        closest_stations_url(BASE_URL, 9.0, 7.0, "petrol"),
        "https://api.example.com/api/v1/stations/closest?latitude=9&longitude=7&station_type=petrol"
    );
    assert_eq!(
        discount_generation_url(BASE_URL),
        "https://api.example.com/api/v1/discounts/generate"
    );
}

#[test]
fn discount_redeem_error_mapping_is_user_friendly() {
    assert_eq!(
        map_discount_redeem_error_message(404, Some("discount code not found")),
        "Invalid code"
    );
    assert_eq!(
        map_discount_redeem_error_message(409, Some("already redeemed")),
        "This discount code has already been redeemed."
    );
    assert_eq!(
        map_discount_redeem_error_message(401, None),
        "Your session may have expired. Please sign in again."
    );
}

#[test]
fn discount_generation_error_mapping_is_user_friendly() {
    assert_eq!(
        map_discount_generation_error_message(429, None),
        "Too many attempts. Please wait a bit and try again."
    );
    assert_eq!(
        map_discount_generation_error_message(500, None),
        "The server is busy right now. Please try again in a moment."
    );
    assert_eq!(
        map_discount_generation_error_message(400, Some("discount is not enabled")),
        "Discounts are not available for this station right now."
    );
}

#[test]
fn stations_api_error_parser_handles_json_and_fallback() {
    let json_body = r#"{"message":"Invalid: token "}"#;
    assert_eq!(parse_api_error_message(401, json_body), "Invalid: token ");

    assert_eq!(
        parse_api_error_message(500, "this-is-not-json"),
        "Server error: 500"
    );
}

#[test]
fn stations_generic_server_error_format_is_stable() {
    assert_eq!(map_status_server_error(403), "Server error: 403");
}
