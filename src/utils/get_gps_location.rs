use wasm_bindgen::prelude::*;
use web_sys::{window, Position, PositionOptions};
use futures::channel::oneshot;
use leptos::logging;

/// Attempts to obtain the user's current latitude/longitude pair.
///
/// # Errors
///
/// Returns an `Err(String)` describing why the location could not be
/// retrieved.  Common reasons include the browser not supporting
/// geolocation, the call failing, or the user denying permission.
pub async fn locate() -> Result<(f64, f64), String> {
    let window = window().ok_or_else(|| "window object unavailable".to_string())?;
    let navigator = window.navigator();
    let geolocation = navigator
        .geolocation()
        .map_err(|_| "Geolocation API unavailable".to_string())?;

    // send a Result through the channel so callers know why we failed
    let (tx, rx) = oneshot::channel::<Result<(f64, f64), String>>();
    
    // Setup Options: Mobile GPS can be slow, so we set a 10s timeout
    let options = PositionOptions::new();
    options.set_enable_high_accuracy(true); 
    options.set_timeout(10000); // 10 seconds timeout
    options.set_maximum_age(60000); // Allow 1-minute old cached location for speed

    // share the sender across both callbacks; once one of them runs we drop it
    use std::rc::Rc;
    use std::cell::RefCell;
    let sender = Rc::new(RefCell::new(Some(tx)));

    let success_sender = sender.clone();
    let success_callback = Closure::wrap(Box::new(move |pos: Position| {
        let coords = pos.coords();
        let lat = coords.latitude();
        let lon = coords.longitude();
        
        logging::log!("GPS Fixed: {}, {}", lat, lon);

        if let Some(s) = success_sender.borrow_mut().take() {
            let _ = s.send(Ok((lat, lon)));
        }
    }) as Box<dyn FnMut(Position)>);

    let error_sender = sender.clone();
    let error_callback = Closure::wrap(Box::new(move |err: JsValue| {
        let msg = err.as_string().unwrap_or_else(|| format!("{:?}", err));
        logging::error!("Geolocation Error: {}", msg);
        if let Some(s) = error_sender.borrow_mut().take() {
            let _ = s.send(Err(format!("Geolocation failed: {}", msg)));
        }
    }) as Box<dyn FnMut(JsValue)>);

    if let Err(_e) = geolocation.get_current_position_with_error_callback_and_options(
        success_callback.as_ref().unchecked_ref(),
        Some(error_callback.as_ref().unchecked_ref()),
        &options,
    ) {
        // If the synchronous call failed (e.g. security policy), send an error
        if let Some(s) = sender.borrow_mut().take() {
            let _ = s.send(Err("Failed to initiate geolocation request".to_string()));
        }
    }

    // Prevent the browser from cleaning up the closures prematurely
    success_callback.forget();
    error_callback.forget();

    match rx.await {
        Ok(res) => res,
        Err(_) => Err("Geolocation channel dropped".to_string()),
    }
}