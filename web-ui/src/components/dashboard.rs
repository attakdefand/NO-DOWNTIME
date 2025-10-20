use yew::prelude::*;
use gloo_timers::callback::Interval;
use wasm_bindgen_futures::spawn_local;
use crate::services::health::HealthService;
use crate::services::metrics::MetricsService;

#[derive(Clone, PartialEq)]
struct DashboardData {
    live_status: bool,
    ready_status: bool,
    active_connections: f64,
    total_requests: u64,
    error_count: u64,
}

impl Default for DashboardData {
    fn default() -> Self {
        Self {
            live_status: false,
            ready_status: false,
            active_connections: 0.0,
            total_requests: 0,
            error_count: 0,
        }
    }
}

#[function_component(Dashboard)]
pub fn dashboard() -> Html {
    let data = use_state(|| DashboardData::default());
    let loading = use_state(|| true);
    
    {
        let data = data.clone();
        let loading = loading.clone();
        
        use_effect_with((), move |_| {
            let data = data.clone();
            let loading = loading.clone();
            
            spawn_local(async move {
                // Simulate API calls
                let health_service = HealthService::new();
                let metrics_service = MetricsService::new();
                
                // Update health status
                match health_service.get_live_status().await {
                    Ok(status) => {
                        data.set(DashboardData {
                            live_status: status.status == "ok",
                            ready_status: data.ready_status,
                            active_connections: data.active_connections,
                            total_requests: data.total_requests,
                            error_count: data.error_count,
                        });
                    }
                    Err(_) => {
                        gloo_console::error!("Failed to fetch live status");
                    }
                }
                
                match health_service.get_ready_status().await {
                    Ok(status) => {
                        data.set(DashboardData {
                            live_status: data.live_status,
                            ready_status: status.status == "ok",
                            active_connections: data.active_connections,
                            total_requests: data.total_requests,
                            error_count: data.error_count,
                        });
                    }
                    Err(_) => {
                        gloo_console::error!("Failed to fetch ready status");
                    }
                }
                
                // Update metrics
                match metrics_service.get_active_connections().await {
                    Ok(count) => {
                        data.set(DashboardData {
                            live_status: data.live_status,
                            ready_status: data.ready_status,
                            active_connections: count,
                            total_requests: data.total_requests,
                            error_count: data.error_count,
                        });
                    }
                    Err(_) => {
                        gloo_console::error!("Failed to fetch active connections");
                    }
                }
                
                loading.set(false);
            });
            
            || ()
        });
    }
    
    // Set up interval to refresh data every 5 seconds
    {
        let data = data.clone();
        use_effect_with((), move |_| {
            let data = data.clone();
            
            let interval = Interval::new(5000, move || {
                let data = data.clone();
                spawn_local(async move {
                    let health_service = HealthService::new();
                    let metrics_service = MetricsService::new();
                    
                    // Update health status
                    if let Ok(status) = health_service.get_live_status().await {
                        data.set(DashboardData {
                            live_status: status.status == "ok",
                            ready_status: data.ready_status,
                            active_connections: data.active_connections,
                            total_requests: data.total_requests,
                            error_count: data.error_count,
                        });
                    }
                    
                    if let Ok(status) = health_service.get_ready_status().await {
                        data.set(DashboardData {
                            live_status: data.live_status,
                            ready_status: status.status == "ok",
                            active_connections: data.active_connections,
                            total_requests: data.total_requests,
                            error_count: data.error_count,
                        });
                    }
                    
                    // Update metrics
                    if let Ok(count) = metrics_service.get_active_connections().await {
                        data.set(DashboardData {
                            live_status: data.live_status,
                            ready_status: data.ready_status,
                            active_connections: count,
                            total_requests: data.total_requests,
                            error_count: data.error_count,
                        });
                    }
                });
            });
            
            move || {
                interval.cancel();
            }
        });
    }

    let live_status_class = if data.live_status {
        "status-indicator live"
    } else {
        "status-indicator down"
    };
    
    let ready_status_class = if data.ready_status {
        "status-indicator ready"
    } else {
        "status-indicator down"
    };

    html! {
        <div class="dashboard-container">
            <div class="page-header">
                <h1>{"Dashboard"}</h1>
                <p class="page-subtitle">{"Overview of the No-Downtime Service status and metrics"}</p>
            </div>
            
            if *loading {
                <div class="card">
                    <div class="card-body text-center">
                        <div class="loading-spinner"></div>
                        <p class="mt-3">{"Loading dashboard data..."}</p>
                    </div>
                </div>
            } else {
                <div class="dashboard-grid">
                    // Status Cards
                    <div class="card status-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Service Status"}</h2>
                        </div>
                        <div class="card-body">
                            <div class="status-grid">
                                <div class="status-item">
                                    <div class="status-label">{"Live"}</div>
                                    <div class={live_status_class}></div>
                                    <div class="status-text">
                                        {if data.live_status { "Operational" } else { "Down" }}
                                    </div>
                                </div>
                                <div class="status-item">
                                    <div class="status-label">{"Ready"}</div>
                                    <div class={ready_status_class}></div>
                                    <div class="status-text">
                                        {if data.ready_status { "Ready" } else { "Not Ready" }}
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    // Active Connections
                    <div class="card metric-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Active Connections"}</h2>
                        </div>
                        <div class="card-body text-center">
                            <div class="metric-value">{data.active_connections}</div>
                            <div class="metric-label">{"Current connections"}</div>
                        </div>
                    </div>
                    
                    // Requests
                    <div class="card metric-card">
                        <div class="card-header">
                            <h2 class="card-title">{"HTTP Requests"}</h2>
                        </div>
                        <div class="card-body text-center">
                            <div class="metric-value">{data.total_requests}</div>
                            <div class="metric-label">{"Total requests"}</div>
                        </div>
                    </div>
                    
                    // Errors
                    <div class="card metric-card error-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Errors"}</h2>
                        </div>
                        <div class="card-body text-center">
                            <div class="metric-value error-value">{data.error_count}</div>
                            <div class="metric-label">{"Total errors"}</div>
                        </div>
                    </div>
                </div>
                
                // Charts placeholder
                <div class="card chart-card mt-4">
                    <div class="card-header">
                        <h2 class="card-title">{"Request Metrics"}</h2>
                    </div>
                    <div class="card-body">
                        <div class="chart-placeholder">
                            <div class="chart-content">
                                <span class="chart-icon">{"📊"}</span>
                                <p>{"Request metrics chart would be displayed here"}</p>
                                <small class="text-muted">{"Real-time data visualization of service performance"}</small>
                            </div>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}