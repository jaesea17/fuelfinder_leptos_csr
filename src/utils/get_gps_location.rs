use wasm_bindgen::prelude::*;
use web_sys::{window, Position};
use futures::channel::oneshot;
use leptos::logging;

pub async fn locate() -> Option<(f64, f64)> {
    let window = window()?;
    let navigator = window.navigator();
    let geolocation = navigator.geolocation().ok()?;

    let (tx, rx) = oneshot::channel::<(f64, f64)>();
    
    // Wrap tx in an Option so we can "take" it out inside an FnMut closure
    let mut tx_opt = Some(tx);

    let success_callback = Closure::wrap(Box::new(move |pos: Position| {
        let coords = pos.coords();
        let lat = coords.latitude();
        let lon = coords.longitude();
        
        logging::log!("latitude: {}, longitude: {}", lat, lon);

        // .take() extracts the value from the Option, leaving None behind
        if let Some(sender) = tx_opt.take() {
            let _ = sender.send((lat, lon));
        }
    }) as Box<dyn FnMut(Position)>);

    // Error callback needs to be handled similarly if it uses the sender
    let error_callback = Closure::wrap(Box::new(move |err: JsValue| {
        logging::error!("Geolocation Error: {:?}", err);
    }) as Box<dyn FnMut(JsValue)>);

    let _ = geolocation.get_current_position_with_error_callback(
        success_callback.as_ref().unchecked_ref(),
        Some(error_callback.as_ref().unchecked_ref()),
    );

    // Memory management: keep closures alive for the browser
    success_callback.forget();
    error_callback.forget();

    rx.await.ok()
}