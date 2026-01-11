use crate::{pages::stations::dto::{RegisterFormData, register_station}, utils::get_gps_location::locate};
use leptos::{logging, prelude::*};
use wasm_bindgen::JsCast;
use leptos_router::{components::{A, Form}, hooks::use_navigate};

#[component]
pub fn Signup() -> impl IntoView {
    // 1. Reactive state for errors and server messages
    let server_error = RwSignal::new(None::<String>);
    let validation_errors = RwSignal::new(std::collections::HashMap::<String, String>::new());
    let navigate = use_navigate();
    
    // 2. The Registration Action
    // We use Action::new_local because we are doing client-side GPS work first
    let register_action = Action::new_local(move |data: &RegisterFormData| {
        let data = data.clone();
        let navigate = navigate.clone();
        
        async move {
            // Step A: Get GPS Location
            let (lat, lon) = match locate().await {
                Some(coords) => coords,
                None => return Err("GPS location is required to register a station.".to_string()),
            };
            let _station = register_station(data, lat, lon).await?;
            logging::log!("Registering at: {}, {}", lat, lon);
            navigate("/signin", Default::default());
            
            Ok("Success".to_string())
        }
    });

    // 3. Form Validation Logic
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        // Extract data from the form element
        let form_data = web_sys::FormData::new_with_form(&ev.target().unwrap().unchecked_into())
            .expect("Failed to get form data");
        
        let name = form_data.get("name").as_string().unwrap_or_default();
        let phone = form_data.get("phone").as_string().unwrap_or_default();
        let address = form_data.get("address").as_string().unwrap_or_default();
        let email = form_data.get("email").as_string().unwrap_or_default();
        let password = form_data.get("password").as_string().unwrap_or_default();
        
        let mut errors = std::collections::HashMap::new();

        // Validations
        if name.is_empty() { errors.insert("name".into(), "Name is required".into()); }
        if address.is_empty() { errors.insert("address".into(), "Address is required".into()); }
        if email.is_empty() { errors.insert("email".into(), "Email is required".into()); }
        if password.is_empty() { errors.insert("password".into(), "Password is required".into()); }
        if phone.len() != 11 || !phone.chars().all(|c| c.is_ascii_digit()){ errors.insert("phone".into(), "Invalid phone number".into()); }

        if errors.is_empty() {
            validation_errors.set(errors);
            let data = RegisterFormData {
                name,
                address,
                email,
                phone,
                password,
            };
            register_action.dispatch(data);
        } else {
            validation_errors.set(errors);
        }
    };

    view! {
        <div class="form-container">
        // ... rest of your form
            <h4>"Note: Please make sure to register at the exact location of your station. As we automatically save the GPS location."</h4>
            <h2>"Register Filling Station"</h2>

            <form on:submit=on_submit>
                <div class="form-group">
                    <label>"Name"</label>
                    <input type="text" name="name" 
                        class=move || validation_errors.with(|e| if e.contains_key("name") { "error" } else { "" }) 
                    />
                    {move || validation_errors.get().get("name").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <div class="form-group">
                    <label>"Address"</label>
                    <input type="text" name="address" />
                    {move || validation_errors.get().get("address").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <div class="form-group">
                    <label>"Email"</label>
                    <input type="email" name="email" autocomplete="email"/>
                    {move || validation_errors.get().get("email").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <div class="form-group">
                    <label>"Phone"</label>
                    <input type="text" name="phone" autocomplete="tel"/>
                    {move || validation_errors.get().get("phone").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <div class="form-group">
                    <label>"Password"</label>
                    <input type="password" name="password" />
                    {move || validation_errors.get().get("password").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <button type="submit" class="submit-button" disabled=move || register_action.pending().get()>
                    {move || if register_action.pending().get() { "Registering..." } else { "Register" }}
                </button>
            </form>

            <p>
                "Already registered? " 
                <A href="/signin">"Login"</A> 
            </p>
            // Server Error Display
            {move || register_action.value().get().and_then(|res| res.err()).map(|err| view! {
                <small class="error-message">{err}</small>
            })}
        </div>
    }
        
}