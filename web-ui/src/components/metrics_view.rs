use yew::prelude::*;
use crate::services::metrics::{MetricsService, Metric};

#[function_component(MetricsView)]
pub fn metrics_view() -> Html {
    let metrics_data = use_state(|| Vec::<Metric>::new());
    let loading = use_state(|| true);
    
    {
        let metrics_data = metrics_data.clone();
        let loading = loading.clone();
        
        use_effect_with((), move |_| {
            let metrics_data = metrics_data.clone();
            let loading = loading.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let metrics_service = MetricsService::new();
                
                match metrics_service.get_http_requests_total().await {
                    Ok(data) => metrics_data.set(data),
                    Err(e) => gloo_console::error!(format!("Error fetching metrics: {:?}", e).as_str()),
                }
                
                loading.set(false);
            });
            
            || ()
        });
    }

    html! {
        <div class="metrics-container">
            <div class="page-header">
                <h1>{"Metrics Visualization"}</h1>
                <p class="page-subtitle">{"View and analyze service metrics and performance data"}</p>
            </div>
            
            if *loading {
                <div class="card">
                    <div class="card-body text-center">
                        <div class="loading-spinner"></div>
                        <p class="mt-3">{"Loading metrics data..."}</p>
                    </div>
                </div>
            } else {
                <div class="dashboard-grid">
                    // HTTP Requests Chart
                    <div class="card metrics-card">
                        <div class="card-header">
                            <h2 class="card-title">{"HTTP Requests"}</h2>
                            <div class="card-actions">
                                <button class="btn btn-outline btn-sm mr-2">{"Refresh"}</button>
                                <button class="btn btn-outline btn-sm">{"Export"}</button>
                            </div>
                        </div>
                        <div class="card-body">
                            <div class="chart-placeholder">
                                <div class="chart-content">
                                    <span class="chart-icon">{"📊"}</span>
                                    <p>{"HTTP requests chart would be displayed here"}</p>
                                    <small class="text-muted">{"Real-time visualization of HTTP request patterns"}</small>
                                </div>
                            </div>
                            if !metrics_data.is_empty() {
                                <div class="table-responsive mt-4">
                                    <table class="table">
                                        <thead>
                                            <tr>
                                                <th>{"Method"}</th>
                                                <th>{"Endpoint"}</th>
                                                <th>{"Status"}</th>
                                                <th>{"Count"}</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {for metrics_data.iter().map(|metric| {
                                                html! {
                                                    <tr>
                                                        <td>{metric.labels.get("method").unwrap_or(&"N/A".to_string())}</td>
                                                        <td>{metric.labels.get("endpoint").unwrap_or(&"N/A".to_string())}</td>
                                                        <td>{metric.labels.get("status").unwrap_or(&"N/A".to_string())}</td>
                                                        <td>{metric.value}</td>
                                                    </tr>
                                                }
                                            })}
                                        </tbody>
                                    </table>
                                </div>
                            }
                        </div>
                    </div>
                    
                    // Request Duration
                    <div class="card metrics-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Request Duration"}</h2>
                        </div>
                        <div class="card-body">
                            <div class="chart-placeholder">
                                <div class="chart-content">
                                    <span class="chart-icon">{"⏱️"}</span>
                                    <p>{"Request duration histogram would be displayed here"}</p>
                                    <small class="text-muted">{"Distribution of request processing times"}</small>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    // Error Distribution
                    <div class="card metrics-card">
                        <div class="card-header">
                            <h2 class="card-title">{"Error Distribution"}</h2>
                        </div>
                        <div class="card-body">
                            <div class="chart-placeholder">
                                <div class="chart-content">
                                    <span class="chart-icon">{"⚠️"}</span>
                                    <p>{"Error distribution chart would be displayed here"}</p>
                                    <small class="text-muted">{"Breakdown of error types and frequencies"}</small>
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
                            <div class="metric-value">{"42"}</div>
                            <div class="metric-label">{"Current connections"}</div>
                        </div>
                    </div>
                </div>
                
                // Time Range Selector
                <div class="card time-range-card mt-4">
                    <div class="card-header">
                        <h2 class="card-title">{"Time Range"}</h2>
                    </div>
                    <div class="card-body">
                        <div class="time-range-controls d-flex align-items-center">
                            <label class="mr-2">{"From:"}</label>
                            <input type="datetime-local" class="form-control mr-3" />
                            <label class="mr-2">{"To:"}</label>
                            <input type="datetime-local" class="form-control mr-3" />
                            <button class="btn btn-primary">{"Apply"}</button>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}