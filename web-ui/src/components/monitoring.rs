use yew::prelude::*;
use gloo_timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;

#[function_component(RealTimeMonitoring)]
pub fn real_time_monitoring() -> Html {
    let active = use_state(|| true);
    let notifications = use_state(|| vec![
        "Service started successfully".to_string(),
        "Health check passed".to_string(),
        "New connection established".to_string(),
    ]);
    let connection_status = use_state(|| "Connected".to_string());
    
    // Simulate real-time data updates
    {
        let notifications = notifications.clone();
        let connection_status = connection_status.clone();
        
        use_effect_with((), move |_| {
            let notifications = notifications.clone();
            let connection_status = connection_status.clone();
            
            let interval = Interval::new(5000, move || {
                let notifications = notifications.clone();
                let connection_status = connection_status.clone();
                
                spawn_local(async move {
                    // Simulate adding new notifications
                    let mut new_notifications = (*notifications).clone();
                    new_notifications.insert(0, format!("New event at {}", 
                        js_sys::Date::now()));
                    
                    // Keep only the last 10 notifications
                    if new_notifications.len() > 10 {
                        new_notifications.truncate(10);
                    }
                    
                    notifications.set(new_notifications);
                    
                    // Randomly change connection status for demonstration
                    let statuses = ["Connected", "Connecting", "Disconnected"];
                    let random_status = statuses[fastrand::usize(..statuses.len())];
                    connection_status.set(random_status.to_string());
                });
            });
            
            move || {
                interval.cancel();
            }
        });
    }
    
    let toggle_monitoring = {
        let active = active.clone();
        Callback::from(move |_| {
            active.set(!*active);
        })
    };

    html! {
        <div>
            <h1>{"Real-time Monitoring"}</h1>
            <p class="mb-4">{"Monitor service activity and receive real-time updates"}</p>
            
            <div class="dashboard-grid">
                // Connection Status
                <div class="card">
                    <div class="card-header">
                        <h2 class="card-title">{"Connection Status"}</h2>
                        <button class="btn btn-outline" onclick={toggle_monitoring}>
                            {if *active { "Pause" } else { "Resume" }}
                        </button>
                    </div>
                    <div class="text-center mt-3">
                        <div class="metric-value">
                            <span class={if &*connection_status == "Connected" { "status-indicator live" } else { "status-indicator down" }}></span>
                            {&*connection_status}
                        </div>
                        <div class="metric-label mt-2">
                            {"WebSocket connection"}
                        </div>
                    </div>
                </div>
                
                // Active Connections
                <div class="card">
                    <div class="card-header">
                        <h2 class="card-title">{"Active Connections"}</h2>
                    </div>
                    <div class="text-center">
                        <div class="metric-value">{"42"}</div>
                        <div class="metric-label">{"Current connections"}</div>
                    </div>
                </div>
                
                // Requests per Second
                <div class="card">
                    <div class="card-header">
                        <h2 class="card-title">{"Requests/Second"}</h2>
                    </div>
                    <div class="text-center">
                        <div class="metric-value">{"12.5"}</div>
                        <div class="metric-label">{"Average RPS"}</div>
                    </div>
                </div>
            </div>
            
            // Notifications Panel
            <div class="card mt-4">
                <div class="card-header">
                    <h2 class="card-title">{"Recent Notifications"}</h2>
                </div>
                <div class="mt-3">
                    if notifications.is_empty() {
                        <p>{"No recent notifications"}</p>
                    } else {
                        <ul>
                            {for notifications.iter().map(|notification| {
                                html! {
                                    <li class="mb-2 p-2" style="border-left: 3px solid var(--primary-color);">
                                        {notification}
                                    </li>
                                }
                            })}
                        </ul>
                    }
                </div>
            </div>
            
            // Live Metrics Chart
            <div class="card mt-4">
                <div class="card-header">
                    <h2 class="card-title">{"Live Metrics"}</h2>
                </div>
                <div class="chart-container">
                    <p>{"Live metrics chart would be displayed here with real-time updates"}</p>
                </div>
            </div>
        </div>
    }
}