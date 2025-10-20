use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::{
    dashboard::Dashboard, 
    health_monitor::HealthMonitor,
    metrics_view::MetricsView,
    auth_panel::AuthPanel,
    config_panel::ConfigPanel,
    monitoring::RealTimeMonitoring
};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Dashboard,
    #[at("/health")]
    Health,
    #[at("/metrics")]
    Metrics,
    #[at("/auth")]
    Auth,
    #[at("/config")]
    Config,
    #[at("/monitoring")]
    Monitoring,
}

impl Route {
    fn title(&self) -> &'static str {
        match self {
            Route::Dashboard => "Dashboard",
            Route::Health => "Health Monitoring",
            Route::Metrics => "Metrics",
            Route::Auth => "Authentication",
            Route::Config => "Configuration",
            Route::Monitoring => "Real-time Monitoring",
        }
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Dashboard => html! { <Dashboard /> },
        Route::Health => html! { <HealthMonitor /> },
        Route::Metrics => html! { <MetricsView /> },
        Route::Auth => html! { <AuthPanel /> },
        Route::Config => html! { <ConfigPanel /> },
        Route::Monitoring => html! { <RealTimeMonitoring /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div id="app">
            <BrowserRouter>
                <Header />
                <main class="container">
                    <Switch<Route> render={switch} />
                </main>
            </BrowserRouter>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct HeaderProps {
    #[prop_or_default]
    on_toggle_theme: Callback<()>,
}

#[function_component(Header)]
fn header() -> Html {
    let navigator = use_navigator().unwrap();
    
    let onclick_dashboard = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Dashboard))
    };
    
    let onclick_health = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Health))
    };
    
    let onclick_metrics = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Metrics))
    };
    
    let onclick_auth = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Auth))
    };
    
    let onclick_config = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Config))
    };
    
    let onclick_monitoring = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::Monitoring))
    };

    html! {
        <header class="header">
            <div class="container header-content">
                <a class="logo" href="/">{"No-Downtime Dashboard"}</a>
                <nav>
                    <div class="nav-links">
                        <a class="nav-link" 
                           onclick={onclick_dashboard}>
                            {"Dashboard"}
                        </a>
                        <a class="nav-link" 
                           onclick={onclick_health}>
                            {"Health"}
                        </a>
                        <a class="nav-link" 
                           onclick={onclick_metrics}>
                            {"Metrics"}
                        </a>
                        <a class="nav-link" 
                           onclick={onclick_auth}>
                            {"Auth"}
                        </a>
                        <a class="nav-link" 
                           onclick={onclick_config}>
                            {"Config"}
                        </a>
                        <a class="nav-link" 
                           onclick={onclick_monitoring}>
                            {"Monitoring"}
                        </a>
                    </div>
                </nav>
            </div>
        </header>
    }
}