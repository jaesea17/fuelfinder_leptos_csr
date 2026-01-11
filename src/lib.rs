use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_navigate;
use leptos_router::{StaticSegment, components::*, path};

use crate::pages::not_found::NotFound;

// Modules
mod components;
mod pages;
mod utils;

// Top-Level pages
use crate::pages::home::Home;
use crate::pages::stations::dashboard::dashboard::StationDashboard;
use crate::pages::stations::signin::Signin;
use crate::pages::stations::signup::Signup;
use crate::utils::protect_route::is_authenticated;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

        // sets the document title
        <Title text="Welcome to Leptos CSR" />

        // connect style for tailwind
        <Stylesheet id="leptos" href="./style/output.css"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Router>
            <img class="logo" src="assets/petrol_pump/pump_red.jpeg" alt="FuelFinder Logo"  />
            <Routes fallback=|| view! { <NotFound/> }>
                <Route path=StaticSegment("/") view=Home/>
                <Route path=StaticSegment("/signup") view=Signup/>
                <Route path=StaticSegment("/signin") view=Signin/>
                <Route 
                    path=StaticSegment("/station") 
                    view=move || {
                        if is_authenticated() {
                            view! { <StationDashboard/> }.into_any()
                        } else {
                            let navigate = use_navigate();
                            navigate("/signin", Default::default());
                            view! { <p>"Redirecting..."</p> }.into_any()
                        }
                    }
                />
               
            </Routes>
        </Router>
    }
}
