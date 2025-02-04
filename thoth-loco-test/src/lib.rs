pub mod app;

use crate::app::App;
use loco_rs::{
    app::Hooks,
    boot::{start, ServeParams, StartMode},
    environment::resolve_from_env,
    logger, Result,
};

#[tokio::main]
pub async fn start_server() -> Result<()> {
    start_app::<App>().await
}

pub async fn start_app<H: Hooks>() -> Result<()> {
    let environment = resolve_from_env().into();
    let config = H::load_config(&environment).await?;

    if !H::init_logger(&config, &environment)? {
        logger::init::<H>(&config.logger)?;
    }

    let boot_result = H::boot(StartMode::WorkerOnly, &environment, config).await?;
    let serve_params = ServeParams {
        port: boot_result.app_context.config.server.port,
        binding: boot_result.app_context.config.server.binding.to_string(),
    };
    start::<H>(boot_result, serve_params, false).await
}
