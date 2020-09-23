use std::io;

use actix_cors::Cors;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use dotenv::dotenv;

macro_rules! static_files {
    ($(($cname:ident, $fname:ident) => ($source_path:expr, $dest_path:expr, $type:expr),)*) => (
        $(
            const $cname: &[u8] = include_bytes!($source_path);

            #[get($dest_path)]
            async fn $fname() -> HttpResponse {
                HttpResponse::Ok().content_type($type).body($cname)
            }
        )*

        fn config(cfg: &mut web::ServiceConfig) {
            dotenv().ok();
            $(cfg.service($fname);)*
        }

    )
}

static_files! {
    (JS, js_file) => ("../static/thoth_manager.js", "/thoth_manager.js", "application/javascript"),
    (WASM, wasm_file) => ("../static/thoth_manager_bg.wasm", "/thoth_manager_bg.wasm", "application/wasm"),
    (PKG, pkg_file) => ("../static/package.json", "/package.json", "application/json"),
    (TS1, ts1_file) => ("../static/thoth_manager.d.ts", "/thoth_manager.d.ts", "application/typescript"),
    (TS2, ts2_file) => ("../static/thoth_manager_bg.d.ts", "/thoth_manager_bg.d.ts", "application/typescript"),
    (MATOMO, matomo_file) => ("../static/js/matomo.js", "/matomo.js", "application/javascript"),
    (BULMA, bulma_file) => ("../static/css/bulma-pageloader.min.css", "/css/bulma-pageloader.min.css", "text/css; charset=utf-8"),
    (CSS, css_file) => ("../static/css/thoth.css", "/css/thoth.css", "text/css; charset=utf-8"),
    (ICON, icon_file) => ("../static/favicon.ico", "/favicon.ico", "image/x-icon"),
    (LOGO, logo_file) => ("../static/img/thoth-logo.png", "/img/thoth-logo.png", "image/png"),
}

const INDEX_FILE: &[u8] = include_bytes!("../static/index.html");

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_FILE)
}

#[actix_rt::main]
pub async fn start_server(port: String) -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .finish(),
            )
            .configure(config)
            .default_service(web::route().to(index))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
