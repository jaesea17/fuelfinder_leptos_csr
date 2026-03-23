use fuelfinder_client::utils::{
    base_url::BaseUrl,
    validate_boundary::{
        ABUJA_MAX_LAT,
        ABUJA_MAX_LON,
        ABUJA_MIN_LAT,
        ABUJA_MIN_LON,
        validate_abuja_bounds,
    },
};

#[test]
fn abuja_boundary_accepts_center_point() {
    assert!(validate_abuja_bounds(9.0, 7.0).is_ok());
}

#[test]
fn abuja_boundary_accepts_exact_edges() {
    assert!(validate_abuja_bounds(ABUJA_MIN_LAT, ABUJA_MIN_LON).is_ok());
    assert!(validate_abuja_bounds(ABUJA_MAX_LAT, ABUJA_MAX_LON).is_ok());
}

#[test]
fn abuja_boundary_rejects_points_outside_range() {
    assert!(validate_abuja_bounds(0.0, 0.0).is_err());
    assert!(validate_abuja_bounds(ABUJA_MAX_LAT + 0.01, 7.0).is_err());
    assert!(validate_abuja_bounds(9.0, ABUJA_MIN_LON - 0.01).is_err());
}

#[test]
fn abuja_boundary_returns_human_readable_error() {
    let error = validate_abuja_bounds(6.5, 3.3).expect_err("coords should be rejected");
    assert_eq!(error, "Location is outside Abuja service area");
}

#[test]
fn base_url_defaults_to_empty_string_when_not_baked_in() {
    assert_eq!(BaseUrl::get_base_url(), option_env!("BASE_URL").unwrap_or(""));
}
