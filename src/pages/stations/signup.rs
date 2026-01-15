use crate::{pages::stations::dto::{RegisterFormData, register_station}, utils::{get_gps_location::locate, validate_boundary}};
use leptos::{logging, prelude::*};
use wasm_bindgen::JsCast;
use leptos_router::{components::{A, Form}, hooks::use_navigate};

#[component]
pub fn Signup() -> impl IntoView {
    let server_error = RwSignal::new(None::<String>);
    let validation_errors = RwSignal::new(std::collections::HashMap::<String, String>::new());
    let navigate = use_navigate();
    
    // 1. Reactive state for password visibility
    let show_password = RwSignal::new(false);
    
    let register_action = Action::new_local(move |data: &RegisterFormData| {
        let data = data.clone();
        let navigate = navigate.clone();
        
        async move {
            let (lat, lon) = match locate().await {
                Some(coords) => coords,
                None => return Err("Could determine GPS location.".to_string()),
            };
            let _ = validate_boundary::validate_abuja_bounds(lat, lon)?;
            let _station = register_station(data, lat, lon).await?;
            logging::log!("Registering at: {}, {}", lat, lon);
            navigate("/signin", Default::default());
            
            Ok("Success".to_string())
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let form_data = web_sys::FormData::new_with_form(&ev.target().unwrap().unchecked_into())
            .expect("Failed to get form data");
        
        let name = form_data.get("name").as_string().unwrap_or_default();
        let phone = form_data.get("phone").as_string().unwrap_or_default();
        let address = form_data.get("address").as_string().unwrap_or_default().to_lowercase();
        let email = form_data.get("email").as_string().unwrap_or_default();
        let password = form_data.get("password").as_string().unwrap_or_default();
        let code = form_data.get("code").as_string().unwrap_or_default();
        
        let mut errors = std::collections::HashMap::new();

        if name.is_empty() { errors.insert("name".into(), "Name is required".into()); }
        if address.is_empty() { errors.insert("address".into(), "Address is required".into()); }
        if email.is_empty() { errors.insert("email".into(), "Email is required".into()); }
        if password.is_empty() { errors.insert("password".into(), "Password is required".into()); }
        if phone.len() != 11 || !phone.chars().all(|c| c.is_ascii_digit()){ errors.insert("phone".into(), "Invalid phone number".into()); }
        if code.is_empty() { errors.insert("code".into(), "Code is required".into()); }

        if errors.is_empty() {
            validation_errors.set(errors);
            let data = RegisterFormData {
                name,
                address,
                email,
                phone,
                password,
                code,
            };
            register_action.dispatch(data);
        } else {
            validation_errors.set(errors);
        }
    };

    view! {
        <div class="form-container">
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
                    <div class="password-wrapper">
                        // 2. Dynamic type based on show_password signal
                        <input 
                            type=move || if show_password.get() { "text" } else { "password" } 
                            name="password" 
                            style="width: 100%; padding-right: 40px;"
                        />
                        // 3. Eye Toggle Button
                        <button 
                            type="button" 
                            class="password-toggle"
                            on:click=move |_| show_password.update(|v| *v = !*v)
                        >
                            {move || if show_password.get() { "hide" } else { "show" }}
                        </button>
                    </div>
                    {move || validation_errors.get().get("password").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <div class="form-group">
                    <label>"Registration Code"</label>
                    <input type="text" name="code" />
                    {move || validation_errors.get().get("code").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                <button type="submit" class="submit-button" disabled=move || register_action.pending().get()>
                    {move || if register_action.pending().get() { "Registering..." } else { "Register" }}
                </button>
            </form>

            <p>
                "Already registered? " 
                <A href="/signin">"Login"</A> 
            </p>
            {move || register_action.value().get().and_then(|res| res.err()).map(|err| view! {
                <small class="error-message">"Oops! an error occurred: "{err}</small>
            })}
        </div>
    }
}