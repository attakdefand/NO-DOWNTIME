use yew::prelude::*;
use web_sys;
use gloo_console;

#[derive(Clone, PartialEq)]
struct ConfigData {
    bind_address: String,
    tls_enabled: bool,
    oauth2_enabled: bool,
    rbac_enabled: bool,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:3000".to_string(),
            tls_enabled: false,
            oauth2_enabled: true,
            rbac_enabled: true,
        }
    }
}

#[function_component(ConfigPanel)]
pub fn config_panel() -> Html {
    let config = use_state(|| ConfigData::default());
    let saved = use_state(|| false);
    
    let on_bind_address_change = {
        let config = config.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            let value = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            let mut new_config = (*config).clone();
            new_config.bind_address = value;
            config.set(new_config);
        })
    };
    
    let on_tls_toggle = {
        let config = config.clone();
        Callback::from(move |_| {
            let mut new_config = (*config).clone();
            new_config.tls_enabled = !new_config.tls_enabled;
            config.set(new_config);
        })
    };
    
    let on_oauth2_toggle = {
        let config = config.clone();
        Callback::from(move |_| {
            let mut new_config = (*config).clone();
            new_config.oauth2_enabled = !new_config.oauth2_enabled;
            config.set(new_config);
        })
    };
    
    let on_rbac_toggle = {
        let config = config.clone();
        Callback::from(move |_| {
            let mut new_config = (*config).clone();
            new_config.rbac_enabled = !new_config.rbac_enabled;
            config.set(new_config);
        })
    };
    
    let on_save = {
        let saved = saved.clone();
        Callback::from(move |_| {
            // In a real implementation, this would save to the backend
            saved.set(true);
            gloo_console::log!("Configuration saved");
        })
    };

    html! {
        <div>
            <h1>{"Configuration"}</h1>
            <p class="mb-4">{"Manage service configuration and settings"}</p>
            
            <div class="card">
                <div class="card-header">
                    <h2 class="card-title">{"Service Configuration"}</h2>
                </div>
                <div class="mt-3">
                    <div class="form-group">
                        <label class="form-label">{"Bind Address"}</label>
                        <input 
                            type="text" 
                            class="form-control" 
                            value={config.bind_address.clone()}
                            oninput={on_bind_address_change}
                        />
                    </div>
                    
                    <div class="form-group">
                        <div class="d-flex justify-content-between align-items-center">
                            <label class="form-label">{"TLS Enabled"}</label>
                            <label class="switch">
                                <input 
                                    type="checkbox" 
                                    checked={config.tls_enabled}
                                    onclick={on_tls_toggle}
                                />
                                <span class="slider"></span>
                            </label>
                        </div>
                    </div>
                    
                    <div class="form-group">
                        <div class="d-flex justify-content-between align-items-center">
                            <label class="form-label">{"OAuth2 Enabled"}</label>
                            <label class="switch">
                                <input 
                                    type="checkbox" 
                                    checked={config.oauth2_enabled}
                                    onclick={on_oauth2_toggle}
                                />
                                <span class="slider"></span>
                            </label>
                        </div>
                    </div>
                    
                    <div class="form-group">
                        <div class="d-flex justify-content-between align-items-center">
                            <label class="form-label">{"RBAC Enabled"}</label>
                            <label class="switch">
                                <input 
                                    type="checkbox" 
                                    checked={config.rbac_enabled}
                                    onclick={on_rbac_toggle}
                                />
                                <span class="slider"></span>
                            </label>
                        </div>
                    </div>
                    
                    <button class="btn btn-primary" onclick={on_save}>
                        {"Save Configuration"}
                    </button>
                    
                    if *saved {
                        <div class="alert alert-success mt-3">
                            {"Configuration saved successfully!"}
                        </div>
                    }
                </div>
            </div>
            
            // TLS Configuration
            <div class="card mt-4">
                <div class="card-header">
                    <h2 class="card-title">{"TLS Configuration"}</h2>
                </div>
                <div class="mt-3">
                    if config.tls_enabled {
                        <div>
                            <div class="form-group">
                                <label class="form-label">{"Certificate File"}</label>
                                <input type="text" class="form-control" placeholder="path/to/cert.pem" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">{"Private Key File"}</label>
                                <input type="text" class="form-control" placeholder="path/to/key.pem" />
                            </div>
                        </div>
                    } else {
                        <p>{"TLS is currently disabled. Enable TLS to configure certificate settings."}</p>
                    }
                </div>
            </div>
            
            // RBAC Configuration
            <div class="card mt-4">
                <div class="card-header">
                    <h2 class="card-title">{"RBAC Configuration"}</h2>
                </div>
                <div class="mt-3">
                    if config.rbac_enabled {
                        <div>
                            <h3>{"Roles"}</h3>
                            <ul>
                                <li>{"admin - Full access to all features"}</li>
                                <li>{"user - Standard access to service features"}</li>
                                <li>{"guest - Read-only access"}</li>
                            </ul>
                            
                            <h3 class="mt-3">{"Permissions"}</h3>
                            <ul>
                                <li>{"read - Read access to service data"}</li>
                                <li>{"write - Write access to service data"}</li>
                                <li>{"admin - Administrative privileges"}</li>
                            </ul>
                        </div>
                    } else {
                        <p>{"RBAC is currently disabled. Enable RBAC to configure role-based access control."}</p>
                    }
                </div>
            </div>
        </div>
    }
}