#![recursion_limit = "512"]

mod components;

use wasm_bindgen::prelude::*;
use crate::components::root::RootComponent;
use yew::prelude::*;

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<RootComponent>::new().mount_to_body();
}
