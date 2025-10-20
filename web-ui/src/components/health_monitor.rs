use yew::prelude::*;
use crate::services::health::{HealthService, HealthStatus};

#[function_component(HealthMonitor)]
pub fn health_monitor() -> Html {
    let live_status = use_state(|| None::<HealthStatus>);
    let ready_status = use_state(|| None::<HealthStatus>);
    let loading = use_state(|| true);
    
    {
        let live_status = live_status.clone();
        let ready_status = ready_status.clone();
        let loading = loading.clone();
        
        use_effect_with((), move |_| {
            let live_status = live_status.clone();
            let ready_status = ready_status.clone();
            let loading = loading.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let health_service = HealthService::new();
                
                match health_service.get_live_status().await {
                    Ok(status) => live_status.set(Some(status)),
                    Err(e) => gloo_console::error!(format!("Error fetching live status: {:?}", e).as_str()),
                }
                
                match health_service.get_ready_status().await {
                    Ok(status) => ready_status.set(Some(status)),
                    Err(e) => gloo_console::error!(format!("Error fetching ready status: {:?}", e).as_str()),
                }
                
                loading.set(false);
            });
            
            || ()
        });
    }
    
    let toggle_live_status = {
        Callback::from(move |_| {
            gloo_console::log!("Toggle live status clicked");
        })
    };
    
    let toggle_ready_status = {
        Callback::from(move |_| {
            gloo_console::log!("Toggle ready status clicked");
        })
    };

    html! {
        <div class="health-monitor-container">
            <div class="page-header">
                <h1>{"Health Monitoring"}</h1>
                <p class="page-subtitle">{"Monitor the health status of the No-Downtime Service"}</p>
            </div>
            
            if *loading {
                <div class="card">
                    <div class="card-body text-center">
                        <div class="loading-spinner"></div>
                        <p class="mt-3">{"Loading health status..."}</p>
                    </div>
                </div>
            } else {
                <div class="dashboard-grid">
                    // Live Status Card
                    <div class="card health-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Liveness Status"}</h2>
                            <button class="btn btn-outline btn-sm" onclick={toggle_live_status}>
                                {"Toggle"}
                            </button>
                        </div>
                        if let Some(status) = &*live_status {
                            <div class="card-body">
                                <div class="status-display">
                                    <div class="status-indicator-large">
                                        <span class={if status.status == "ok" { "status-indicator live" } else { "status-indicator down" }}></span>
                                    </div>
                                    <div class="status-text-large">
                                        {if status.status == "ok" { "Operational" } else { "Down" }}
                                    </div>
                                    <div class="status-description">
                                        {"Service is "} 
                                        {if status.status == "ok" { "alive and responding" } else { "not responding" }}
                                    </div>
                                </div>
                                
                                if !status.checks.is_empty() {
                                    <div class="checks-section mt-4">
                                        <h3>{"Health Checks"}</h3>
                                        <div class="checks-list">
                                            {for status.checks.iter().map(|check| {
                                                let status_class = if check.status == "ok" { "check-item success" } else { "check-item error" };
                                                html! {
                                                    <div class={status_class}>
                                                        <div class="check-name">{&check.name}</div>
                                                        <div class="check-status">
                                                            <span class={if check.status == "ok" { "status-indicator ready" } else { "status-indicator down" }}></span>
                                                            {&check.status}
                                                        </div>
                                                        if let Some(message) = &check.message {
                                                            <div class="check-message">{message}</div>
                                                        }
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            </div>
                        }
                    </div>
                    
                    // Ready Status Card
                    <div class="card health-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Readiness Status"}</h2>
                            <button class="btn btn-outline btn-sm" onclick={toggle_ready_status}>
                                {"Toggle"}
                            </button>
                        </div>
                        if let Some(status) = &*ready_status {
                            <div class="card-body">
                                <div class="status-display">
                                    <div class="status-indicator-large">
                                        <span class={if status.status == "ok" { "status-indicator ready" } else { "status-indicator down" }}></span>
                                    </div>
                                    <div class="status-text-large">
                                        {if status.status == "ok" { "Ready" } else { "Not Ready" }}
                                    </div>
                                    <div class="status-description">
                                        {"Service is "} 
                                        {if status.status == "ok" { "ready to serve requests" } else { "not ready to serve requests" }}
                                    </div>
                                </div>
                                
                                if !status.checks.is_empty() {
                                    <div class="checks-section mt-4">
                                        <h3>{"Readiness Checks"}</h3>
                                        <div class="checks-list">
                                            {for status.checks.iter().map(|check| {
                                                let status_class = if check.status == "ok" { "check-item success" } else { "check-item error" };
                                                html! {
                                                    <div class={status_class}>
                                                        <div class="check-name">{&check.name}</div>
                                                        <div class="check-status">
                                                            <span class={if check.status == "ok" { "status-indicator ready" } else { "status-indicator down" }}></span>
                                                            {&check.status}
                                                        </div>
                                                        if let Some(message) = &check.message {
                                                            <div class="check-message">{message}</div>
                                                        }
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            </div>
                        }
                    </div>
                </div>
                
                // Health History
                <div class="card history-card mt-4">
                    <div class="card-header">
                        <h2 class="card-title">{"Health History"}</h2>
                    </div>
                    <div class="card-body">
                        <div class="table-responsive">
                            <table class="table">
                                <thead>
                                    <tr>
                                        <th>{"Time"}</th>
                                        <th>{"Check"}</th>
                                        <th>{"Status"}</th>
                                        <th>{"Message"}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td>{"2025-10-20 14:30:00"}</td>
                                        <td>{"liveness"}</td>
                                        <td><span class="status-indicator ready"></span>{" ok"}</td>
                                        <td>{"Service is alive"}</td>
                                    </tr>
                                    <tr>
                                        <td>{"2025-10-20 14:30:00"}</td>
                                        <td>{"readiness"}</td>
                                        <td><span class="status-indicator ready"></span>{" ok"}</td>
                                        <td>{"Service is ready"}</td>
                                    </tr>
                                    <tr>
                                        <td>{"2025-10-20 14:25:00"}</td>
                                        <td>{"liveness"}</td>
                                        <td><span class="status-indicator down"></span>{" error"}</td>
                                        <td>{"Timeout occurred"}</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}