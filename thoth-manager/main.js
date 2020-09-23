import init, { run_app } from "./thoth_manager.js";

async function main() {
  await init("/thoth_manager_bg.wasm");
  run_app();
}

main();
