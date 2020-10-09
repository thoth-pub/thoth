#![recursion_limit = "2048"]

use std::env;
use wasm_bindgen::prelude::*;

mod agent;
#[macro_use]
mod component;
mod models;
mod route;
mod string;

use crate::component::root::RootComponent;

pub const API_ENDPOINT: &str = env!("API_ENDPOINT");

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<RootComponent>();
    Ok(())
}
