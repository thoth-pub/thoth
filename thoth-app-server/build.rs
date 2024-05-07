use dotenv::dotenv;
use std::env;
use std::process::{exit, Command};

const TRUNK_VERSION: &str = "0.20.0";

fn is_wasm_target_installed() -> bool {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .expect("Failed to execute rustup");

    let installed_targets = String::from_utf8_lossy(&output.stdout);
    installed_targets.contains("wasm32-unknown-unknown")
}

fn install_wasm_target() {
    println!("Adding wasm32-unknown-unknown target...");
    let output = Command::new("rustup")
        .args(["target", "add", "wasm32-unknown-unknown"])
        .output()
        .expect("Failed to execute rustup");

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }
}

fn get_trunk_version() -> Option<String> {
    Command::new("trunk")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|version_string| version_string.split_whitespace().last().map(String::from))
}

fn install_trunk() -> Result<(), Box<dyn std::error::Error>> {
    println!("Installing trunk {}...", TRUNK_VERSION);

    let output = Command::new("cargo")
        .arg("install")
        .arg("trunk")
        .arg("--version")
        .arg(TRUNK_VERSION)
        .arg("--force")
        .output()?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }

    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=../thoth-app/");
    // load environment variables from `.env`
    dotenv().ok();

    if !is_wasm_target_installed() {
        install_wasm_target();
    }

    if let Some(version) = get_trunk_version() {
        if !version.eq(TRUNK_VERSION) {
            println!("Current trunk version: {}", version);
            install_trunk().unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                exit(1);
            });
        }
    } else {
        println!("trunk not found");
        install_trunk().unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            exit(1);
        });
    }

    // need to change target directory to avoid deadlocking
    env::set_var("CARGO_TARGET_DIR", "../thoth-app/target");
    let mut trunk_command = Command::new("trunk");
    trunk_command.args([
        "build",
        "--config",
        "../thoth-app/Trunk.toml",
        "../thoth-app/index.html",
    ]);

    // Add --release argument if not in debug mode
    if cfg!(not(debug_assertions)) {
        trunk_command.arg("--release");
    }

    let trunk_output = trunk_command.output().expect("Failed to execute trunk");

    if !trunk_output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&trunk_output.stderr));
        exit(1);
    }
    println!("{}", String::from_utf8_lossy(&trunk_output.stdout));
}
