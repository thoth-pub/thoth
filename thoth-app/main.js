import init, { run_app } from "./pkg/thoth_app.js";

async function main() {
  await init("/thoth_app_bg.wasm");
  run_app();
}

main();
