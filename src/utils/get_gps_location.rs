use wasm_bindgen::prelude::*;
use web_sys::{window, Position, PositionOptions}; // Added PositionOptions
use futures::channel::oneshot;
use leptos::logging;

pub async fn locate() -> Option<(f64, f64)> {
    let window = window()?;
    let navigator = window.navigator();
    let geolocation = navigator.geolocation().ok()?;

    let (tx, rx) = oneshot::channel::<(f64, f64)>();
    
    // We need two Options for the sender because we have two separate closures
    let mut tx_success = Some(tx);

    // 1. Setup Options: Mobile GPS can be slow, so we set a 10s timeout
    let options = PositionOptions::new();
    options.set_enable_high_accuracy(true); 
    options.set_timeout(10000); // 10 seconds timeout
    options.set_maximum_age(60000); // Allow 1-minute old cached location for speed

    let success_callback = Closure::wrap(Box::new(move |pos: Position| {
        let coords = pos.coords();
        let lat = coords.latitude();
        let lon = coords.longitude();
        
        logging::log!("GPS Fixed: {}, {}", lat, lon);

        if let Some(sender) = tx_success.take() {
            let _ = sender.send((lat, lon));
        }
    }) as Box<dyn FnMut(Position)>);

    // 2. IMPORTANT: The error callback must exist to unblock the 'rx' channel
    // We use a separate channel or just let rx fail when tx is dropped.
    // However, to be safe, we'll let this closure just log.
    let error_callback = Closure::wrap(Box::new(move |err: JsValue| {
        logging::error!("Geolocation Error or Timeout: {:?}", err);
        // When this closure finishes, if success_callback hasn't run, 
        // tx_success is dropped, which sends a 'Cancel' to rx.await.
    }) as Box<dyn FnMut(JsValue)>);

    let _ = geolocation.get_current_position_with_error_callback_and_options(
        success_callback.as_ref().unchecked_ref(),
        Some(error_callback.as_ref().unchecked_ref()),
        &options // Pass the options here
    );

    // Prevent the browser from cleaning up the closures prematurely
    success_callback.forget();
    error_callback.forget();

    // rx.await.ok() will now return None if:
    // - The user denies permission
    // - The 10-second timeout is reached
    // - The device loses GPS signal
    rx.await.ok()
}