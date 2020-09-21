#![recursion_limit = "2048"]

use wasm_bindgen::prelude::*;

mod agent;
#[macro_use]
mod api;
mod component;
mod route;
mod service;
mod string;

use crate::{component::root::RootComponent, service::log::init_logger};

const GRAPHQL_ENDPOINT: &str = "http://localhost:8000/graphql";

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    init_logger().map_err(|e| JsValue::from(e.to_string()))?;

    yew::start_app::<RootComponent>();
    Ok(())
}
