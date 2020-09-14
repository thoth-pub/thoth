#![recursion_limit = "1024"]

use wasm_bindgen::prelude::*;

mod agent;
#[macro_use]
mod api;
mod component;
mod route;
mod service;
mod string;

use crate::{component::root::RootComponent, service::log::init_logger};

const SESSION_COOKIE: &str = "sessionToken";

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    init_logger().map_err(|e| JsValue::from(e.to_string()))?;

    yew::start_app::<RootComponent>();
    Ok(())
}
