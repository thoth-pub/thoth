use dotenv::dotenv;
use std::{env, fs};

const DOTENV_PATH: &str = "../.env";

/// This build script is responsible for optionally loading environment variables from a `.env` file,
/// setting them in Cargo's environment using `cargo:rustc-env`, and printing them out.
///
/// Simply loading environment variables using `dotenv()` is not sufficient for them to be
/// available during the build process. Hence, they need to be explicitly set in Cargo's
/// environment using `cargo:rustc-env`.
fn main() {
    println!("cargo:rerun-if-changed={DOTENV_PATH}");
    // load environment variables from `.env`
    if dotenv().is_err() {
        println!("No .env file found");
        return;
    }

    // Need to set variables in cargo's environment, otherwise they're only available in this step.
    // Iterate over environment variables and set only those present in the .env file
    let env_file_content = fs::read_to_string(DOTENV_PATH).unwrap();
    for (key, value) in env::vars() {
        if env_file_content.contains(&format!("{key}={value}")) {
            println!("cargo:rustc-env={key}={value}");
        }
    }
}
