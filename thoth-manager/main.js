import init, { run_app } from "./pkg/thoth_manager.js";

async function main() {
  await init("/pkg/thoth_manager_bg.wasm");
  run_app();
}

main();
