use leptos::prelude::*;

#[component]
pub fn Landing() -> impl IntoView {
    view! {
        <div class="landing-container">
            <div class="landing-content">
                <h1>"FuelGetter"</h1>
                
                <div class="landing-cards">
                    <a href="/gas" class="landing-card gas-card">
                        <img src="assets/gas_cylinder/cylinder3.jpeg" alt="Cooking Gas" />
                        <h2>"Cooking Gas Stations"</h2>
                        <p>"Find cooking gas stations near your location"</p>
                    </a>
                    <a href="/petrol" class="landing-card petrol-card">
                        <img src="assets/petrol_pump/pump_red.jpeg" alt="Petrol" />
                        <h2>"Petrol Stations"</h2>
                        <p>"Find petrol stations near your location"</p>
                    </a>
                </div>
            </div>
        </div>
    }
}
