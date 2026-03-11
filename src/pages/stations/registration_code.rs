use crate::pages::stations::dto::generate_reg_code;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;

#[component]
pub fn RegistrationCode() -> impl IntoView {
    let validation_errors = RwSignal::new(std::collections::HashMap::<String, String>::new());
    let generated_code = RwSignal::new(None::<String>);
    let copied = RwSignal::new(false);
    let show_super_password = RwSignal::new(false);
    
    let generate_action = Action::new_local(move |data: &(String, String)| {
        let code = data.0.clone();
        let super_password = data.1.clone();
        
        async move {
            generate_reg_code(code, super_password).await
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        
        let form_data = web_sys::FormData::new_with_form(&ev.target().unwrap().unchecked_into())
            .expect("Failed to get form data");
        
        let code = form_data.get("code").as_string().unwrap_or_default();
        let super_password = form_data.get("super_password").as_string().unwrap_or_default();
        
        let mut errors = std::collections::HashMap::new();

        if code.is_empty() { 
            errors.insert("code".into(), "Code is required".into()); 
        }
        if super_password.is_empty() { 
            errors.insert("super_password".into(), "Super password is required".into()); 
        }
        
        if errors.is_empty() {
            validation_errors.set(errors);
            generate_action.dispatch((code, super_password));
        } else {
            validation_errors.set(errors);
        }
    };

    let copy_to_clipboard = move |_| {
        if let Some(code) = generated_code.get() {
            let window = web_sys::window().unwrap();
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();
            
            let _ = clipboard.write_text(&code);
            copied.set(true);
            
            // Reset copied state after 2 seconds
            set_timeout(
                move || copied.set(false),
                std::time::Duration::from_secs(2),
            );
        }
    };

    Effect::new(move |_| {
        if let Some(Ok(code)) = generate_action.value().get() {
            generated_code.set(Some(code));
        }
    });

    view! {
        <div class="signin-page">
            <div class="form-container">
                <h2>"Generate Registration Code"</h2>
                
                <form on:submit=on_submit>
                    <div class="form-group">
                        <label>"Code"</label>
                        <input type="text" name="code" placeholder="Enter code"/>
                        {move || validation_errors.get().get("code").map(|m| view! { 
                            <small class="error-message">{m.clone()}</small> 
                        })}
                    </div>

                    <div class="form-group">
                        <label>"Super Password"</label>
                        <div class="password-wrapper">
                            <input
                                type=move || if show_super_password.get() { "text" } else { "password" }
                                name="super_password"
                                placeholder="Enter super password"
                                style="width: 100%; padding-right: 40px;"
                            />
                            <button
                                type="button"
                                class="password-toggle"
                                on:click=move |_| show_super_password.update(|v| *v = !*v)
                            >
                                {move || if show_super_password.get() { "hide" } else { "show" }}
                            </button>
                        </div>
                        {move || validation_errors.get().get("super_password").map(|m| view! { 
                            <small class="error-message">{m.clone()}</small> 
                        })}
                    </div>

                    <button type="submit" class="submit-button" disabled=move || generate_action.pending().get()>
                        {move || if generate_action.pending().get() { "Generating..." } else { "Generate Code" }}
                    </button>
                </form>

                {move || generate_action.value().get().and_then(|res| res.err()).map(|err: String| view! {
                    <small class="error-message">{err}</small>
                })}

                {move || generated_code.get().map(|code| view! {
                    <div class="success-container">
                        <p class="success-message">"✓ Code generated successfully!"</p>
                        <div class="code-display">
                            <code class="generated-code">{code.clone()}</code>
                            <button 
                                type="button" 
                                class="copy-button" 
                                on:click=copy_to_clipboard
                            >
                                {move || if copied.get() { "✓ Copied!" } else { "Copy" }}
                            </button>
                        </div>
                    </div>
                })}
            </div>
        </div>
    }
}
