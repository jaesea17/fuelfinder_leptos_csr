use crate::{pages::stations::dto::{LoginFormData, login_station}};
use leptos::{logging, prelude::*};
use wasm_bindgen::JsCast;
use leptos_router::{components::{A, Form}, hooks::use_navigate};

#[component]
pub fn Signin() -> impl IntoView {
    // 1. Reactive state for errors and server messages
    let server_error = RwSignal::new(None::<String>);
    let validation_errors = RwSignal::new(std::collections::HashMap::<String, String>::new());
    let navigate = use_navigate();
    let show_password = RwSignal::new(false);
    
    // 2. The Registration Action
    // We use Action::new_local because we are doing client-side GPS work first
    let login_action = Action::new_local(move |data: &LoginFormData| {
        let payload = data.clone();
        let navigate = navigate.clone();
        
        async move {
            let data = login_station(payload).await?;
            let window = web_sys::window().unwrap();
            let storage = window.local_storage().unwrap().unwrap();
            storage.set_item("accessToken", &data.access_token).unwrap();
            navigate("/station", Default::default());
            
            Ok("Success".to_string())
        }
    });

    // 3. Form Validation Logic
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        // Extract data from the form element
        let form_data = web_sys::FormData::new_with_form(&ev.target().unwrap().unchecked_into())
            .expect("Failed to get form data");
        
        let email = form_data.get("email").as_string().unwrap_or_default();
        let password = form_data.get("password").as_string().unwrap_or_default();
        let station_type = form_data.get("station_type").as_string().unwrap_or_default();
        
        let mut errors = std::collections::HashMap::new();

        // Validations
        if email.is_empty() { errors.insert("email".into(), "Email is required".into()); }
        if password.is_empty() { errors.insert("password".into(), "Password is required".into()); }
        if station_type.is_empty() { errors.insert("station_type".into(), "station type is required".into()); }
        
        if errors.is_empty() {
            validation_errors.set(errors);
            let data = LoginFormData {
                email,
                password,
                station_type,
            };
            login_action.dispatch(data);
        } else {
            validation_errors.set(errors);
        }
    };

    view! {
        <div class="signin-page">
            <div class="form-container">
                <form on:submit=on_submit>
                <div class="form-group">
                    <label>"Email"</label>
                    <input type="email" name="email" autocomplete="email"/>
                    {move || validation_errors.get().get("email").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
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
                    <label>"Station Type"</label>
                    <select name="station_type">
                        <option value="">"-- Select --"</option>
                        <option value="petrol">"Petrol"</option>
                        <option value="gas">"Cooking Gas"</option>
                    </select>
                    {move || validation_errors.get().get("station_type").map(|m| view! { <small class="error-message">{m.clone()}</small> })}
                </div>

                    <button type="submit" class="submit-button" disabled=move || login_action.pending().get()>
                        {move || if login_action.pending().get() { "..." } else { "Login" }}
                    </button>
                </form>

                <p>"Do not have an account? " <A href="/signup">"Register"</A></p>
                // Server Error Display
                {move || login_action.value().get().and_then(|res| res.err()).map(|err: String| {
                    if err.to_lowercase().contains("subscription expired") {
                        view! {
                            <div class="subscription-expired-banner">
                                <span class="sub-expired-icon">"⏰"</span>
                                <div>
                                    <strong>"Subscription Expired"</strong>
                                    <p>"Your subscription has expired. Please contact the administrator to renew it."</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <small class="error-message">{err}</small>
                        }.into_any()
                    }
                })}
            </div>
        </div>
    }
        
}