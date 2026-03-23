mod common;

use fuelfinder_client::pages::{
    admin::dto::{map_admin_http_error, validate_admin_password},
    fetch_nearest_stations_dto::{
        map_status_server_error as nearest_server_error,
        parse_discount_generation_error_message,
        should_retry_closest_request,
    },
    stations::dto::{
        map_discount_redeem_error_message,
        map_status_server_error as stations_server_error,
        parse_api_error_message,
    },
};

#[test]
fn mocked_login_error_parses_json_message() {
    let body = r#"{"message":"Invalid: email or password "}"#;
    let parsed = parse_api_error_message(401, body);
    assert_eq!(parsed, "Invalid: email or password ");
}

#[test]
fn mocked_login_error_falls_back_for_non_json_body() {
    let parsed = parse_api_error_message(500, "not-json");
    assert_eq!(parsed, "Server error: 500");
}

#[test]
fn mocked_admin_http_status_mapping_is_correct() {
    assert_eq!(map_admin_http_error(401), "Invalid admin password");
    assert_eq!(map_admin_http_error(403), "Invalid admin password");
    assert_eq!(map_admin_http_error(500), "Server error: 500");
}

#[test]
fn mocked_admin_password_validation_rejects_blank_values() {
    assert!(validate_admin_password("   ").is_err());
    assert!(validate_admin_password("super-secret").is_ok());
}

#[test]
fn mocked_closest_retry_logic_matches_transient_statuses() {
    assert!(should_retry_closest_request(0, 503));
    assert!(should_retry_closest_request(0, 502));
    assert!(should_retry_closest_request(0, 504));
    assert!(!should_retry_closest_request(1, 503));
    assert!(!should_retry_closest_request(0, 429));
}

#[test]
fn mocked_discount_generation_error_parses_api_json() {
    let body = r#"{"message":"daily discount code limit reached for this station"}"#;
    let message = parse_discount_generation_error_message(400, body);
    assert_eq!(
        message,
        "You’ve already generated a discount code for this station today. Please try again tomorrow."
    );
}

#[test]
fn mocked_discount_generation_error_handles_plain_text() {
    let message = parse_discount_generation_error_message(404, "station not found");
    assert_eq!(
        message,
        "Station details were not found. Please refresh and try again."
    );
}

#[test]
fn mocked_discount_redeem_error_mappings_cover_common_cases() {
    assert_eq!(
        map_discount_redeem_error_message(404, Some("discount code not found")),
        "Invalid code"
    );
    assert_eq!(
        map_discount_redeem_error_message(401, None),
        "Your session may have expired. Please sign in again."
    );
    assert_eq!(
        map_discount_redeem_error_message(409, Some("already redeemed")),
        "This discount code has already been redeemed."
    );
}

#[test]
fn mocked_generic_server_error_formatting_is_consistent() {
    assert_eq!(stations_server_error(404), "Server error: 404");
    assert_eq!(nearest_server_error(429), "Server error: 429");
}
