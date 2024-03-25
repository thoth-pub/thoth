import init, { run_app } from "./pkg/thoth_app.js";

async function main() {
  await init("/admin/thoth_app_bg.wasm");
  run_app();
}

main();
