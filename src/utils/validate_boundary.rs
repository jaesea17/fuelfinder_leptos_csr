pub fn validate_abuja_bounds(lat: f64, lon: f64) -> Result<(), String> {
    let min_lat = 8.25;
    let max_lat = 9.30;
    let min_lon = 6.75;
    let max_lon = 7.75;

    if lat >= min_lat && lat <= max_lat && lon >= min_lon && lon <= max_lon {
        Ok(())
    } else {
        Err("Oops! seems like you are outside the Abuja service area.".into())
    }
}