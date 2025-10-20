use yew::prelude::*;

#[function_component(AuthPanel)]
pub fn auth_panel() -> Html {
    let is_logged_in = use_state(|| false);
    let username = use_state(|| String::new());
    
    let on_login = {
        let is_logged_in = is_logged_in.clone();
        let username = username.clone();
        Callback::from(move |_| {
            is_logged_in.set(true);
            username.set("admin".to_string());
        })
    };
    
    let on_logout = {
        let is_logged_in = is_logged_in.clone();
        let username = username.clone();
        Callback::from(move |_| {
            is_logged_in.set(false);
            username.set(String::new());
        })
    };

    html! {
        <div>
            <h1>{"Authentication"}</h1>
            <p class="mb-4">{"Manage authentication and user access for the No-Downtime Service"}</p>
            
            <div class="dashboard-grid">
                // OAuth2 Login
                <div class="card">
                    <div class="card-header">
                        <h2 class="card-title">{"OAuth2 Authentication"}</h2>
                    </div>
                    <div class="text-center mt-3">
                        if *is_logged_in {
                            <div>
                                <p>{"Logged in as: "}{&*username}</p>
                                <button class="btn btn-outline" onclick={on_logout}>
                                    {"Logout"}
                                </button>
                            </div>
                        } else {
                            <button class="btn btn-primary" onclick={on_login}>
                                {"Login with OAuth2"}
                            </button>
                        }
                    </div>
                </div>
                
                // User Roles
                <div class="card">
                    <div class="card-header">
                        <h2 class="card-title">{"User Roles"}</h2>
                    </div>
                    if *is_logged_in {
                        <div class="mt-3">
                            <ul>
                                <li>{"admin - Full access to all features"}</li>
                                <li>{"user - Standard access to service features"}</li>
                                <li>{"guest - Read-only access"}</li>
                            </ul>
                        </div>
                    } else {
                        <p class="mt-3">{"Login to view user roles"}</p>
                    }
                </div>
            </div>
            
            // Protected Resources
            <div class="card mt-4">
                <div class="card-header">
                    <h2 class="card-title">{"Protected Resources"}</h2>
                </div>
                <div class="mt-3">
                    <ul>
                        <li>
                            <strong>{"/protected"}</strong> 
                            <span class="ml-2">{"- Example protected endpoint"}</span>
                            if *is_logged_in {
                                <span class="ml-2" style="color: var(--success-color);">{"✓ Access granted"}</span>
                            } else {
                                <span class="ml-2" style="color: var(--danger-color);">{"✗ Access denied"}</span>
                            }
                        </li>
                        <li>
                            <strong>{"/admin"}</strong> 
                            <span class="ml-2">{"- Administrative functions"}</span>
                            if *is_logged_in && &*username == "admin" {
                                <span class="ml-2" style="color: var(--success-color);">{"✓ Access granted"}</span>
                            } else {
                                <span class="ml-2" style="color: var(--danger-color);">{"✗ Access denied"}</span>
                            }
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    }
}