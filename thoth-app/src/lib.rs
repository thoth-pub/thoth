#![recursion_limit = "2048"]

use std::env;
use wasm_bindgen::prelude::*;

#[macro_use]
mod agent;
#[macro_use]
mod component;
mod models;
mod route;
mod service;
mod string;

use crate::component::root::RootComponent;

pub const THOTH_GRAPHQL_API: &str = env!("THOTH_GRAPHQL_API");
pub const THOTH_EXPORT_API: &str = env!("THOTH_EXPORT_API");
const SESSION_KEY: &str = "thoth.token";

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<RootComponent>();
    Ok(())
}
