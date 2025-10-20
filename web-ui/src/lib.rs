mod app;
mod components;
mod services;
mod utils;

pub use app::App;

use wasm_bindgen::prelude::*;
use web_sys::{window, Element, console};

#[wasm_bindgen]
pub fn mount_app() -> Result<(), JsValue> {
    console::log_1(&"Mounting Yew application".into());
    
    // Get the window and document
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    
    // Try to get the app element
    match document.get_element_by_id("yew-app") {
        Some(app_element) => {
            console::log_1(&"Found #yew-app element, mounting Yew application".into());
            // Render the app to the specific element
            yew::Renderer::<App>::with_root(app_element).render();
            console::log_1(&"Yew application mounted successfully".into());
            Ok(())
        }
        None => {
            console::log_1(&"ERROR: Could not find #yew-app element".into());
            Err("Could not find #yew-app element".into())
        }
    }
}