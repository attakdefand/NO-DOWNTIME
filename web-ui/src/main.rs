mod app;
mod components;
mod services;
mod utils;

use app::App;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    // Set up logging
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("No-Downtime Service Dashboard WASM module loaded");
    // We'll let JavaScript call mount_app() when it's ready
}
