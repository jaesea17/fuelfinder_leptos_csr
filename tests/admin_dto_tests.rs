mod common;

use fuelfinder_client::pages::admin::dto::{
    admin_stations_url,
    discount_stats_url,
    map_admin_http_error,
    map_admin_request_error_message,
    renew_subscription_url,
    update_discount_url,
    validate_admin_password,
};

use common::{BASE_URL, COMMODITY_ID};

#[test]
fn admin_endpoint_urls_match_router() {
    assert_eq!(
        admin_stations_url(BASE_URL, "all"),
        "https://api.example.com/api/v1/admin/stations?filter=all"
    );
    assert_eq!(
        renew_subscription_url(BASE_URL),
        "https://api.example.com/api/v1/auth/subscriptions/renew"
    );
    assert_eq!(
        update_discount_url(BASE_URL, COMMODITY_ID),
        format!("https://api.example.com/api/v1/admin/discounts/{COMMODITY_ID}")
    );
    assert_eq!(
        discount_stats_url(BASE_URL),
        "https://api.example.com/api/v1/admin/discounts/stats"
    );
}

#[test]
fn admin_network_errors_are_mapped_to_actionable_text() {
    let message = map_admin_request_error_message("TypeError: Failed to fetch".to_string());
    assert!(message.contains("Unable to reach the admin API"));
    assert!(message.contains("X-Admin-Password"));
}

#[test]
fn admin_non_network_errors_are_preserved() {
    let message = map_admin_request_error_message("Server error: 500".to_string());
    assert_eq!(message, "Server error: 500");
}

#[test]
fn admin_http_error_mapping_is_consistent() {
    assert_eq!(map_admin_http_error(401), "Invalid admin password");
    assert_eq!(map_admin_http_error(403), "Invalid admin password");
    assert_eq!(map_admin_http_error(500), "Server error: 500");
}

#[test]
fn admin_password_validation_handles_blank_input() {
    assert!(validate_admin_password("   ").is_err());
    assert!(validate_admin_password("secret").is_ok());
}
