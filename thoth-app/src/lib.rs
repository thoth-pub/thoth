#![recursion_limit = "2048"]

use std::env;
use wasm_bindgen::prelude::*;

mod agent;
#[macro_use]
mod component;
mod models;
mod route;
mod service;
mod string;

use crate::component::root::RootComponent;

pub const THOTH_API: &str = env!("THOTH_API");
const SESSION_COOKIE: &str = "sessionToken";

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<RootComponent>();
    Ok(())
}
