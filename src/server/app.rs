use std::io;

use actix_cors::Cors;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use dotenv::dotenv;

const NO_CACHE: &str = "no-cache";

macro_rules! static_files {
    ($(($cname:ident, $fname:ident) => ($source_path:expr, $dest_path:expr, $type:expr),)*) => (
        $(
            const $cname: &[u8] = include_bytes!($source_path);

            #[get($dest_path)]
            async fn $fname() -> HttpResponse {
                HttpResponse::Ok().content_type($type).header("Cache-Control", NO_CACHE).body($cname)
            }
        )*

        fn config(cfg: &mut web::ServiceConfig) {
            dotenv().ok();
            $(cfg.service($fname);)*
        }

    )
}

static_files! {
    (JS, js_file) => ("../static/pkg/thoth_app.js", "/thoth_app.js", "application/javascript"),
    (WASM, wasm_file) => ("../static/pkg/thoth_app_bg.wasm", "/thoth_app_bg.wasm", "application/wasm"),
    (PKG, pkg_file) => ("../static/pkg/package.json", "/package.json", "application/json"),
    (TS1, ts1_file) => ("../static/pkg/thoth_app.d.ts", "/thoth_app.d.ts", "application/typescript"),
    (TS2, ts2_file) => ("../static/pkg/thoth_app_bg.d.ts", "/thoth_app_bg.d.ts", "application/typescript"),
    (BULMA, bulma_file) => ("../static/css/bulma-pageloader.min.css", "/css/bulma-pageloader.min.css", "text/css; charset=utf-8"),
    (CSS, css_file) => ("../static/css/thoth.css", "/css/thoth.css", "text/css; charset=utf-8"),
    (LOGO, logo_file) => ("../static/img/thoth-logo.png", "/img/thoth-logo.png", "image/png"),
    (XML, xml_file) => ("../static/browserconfig.xml", "/browserconfig.xml", "application/xml"),
    (JSON, json_file) => ("../static/manifest.json", "/manifest.json", "application/json"),
    (ICON, icon_file) => ("../static/img/favicon.ico", "/favicon.ico", "image/x-icon"),
    (ICON1, icon_file1) => ("../static/img/android-icon-144x144.png", "/android-icon-144x144.png", "image/png"),
    (ICON2, icon_file2) => ("../static/img/android-icon-192x192.png", "/android-icon-192x192.png", "image/png"),
    (ICON3, icon_file3) => ("../static/img/android-icon-36x36.png", "/android-icon-36x36.png", "image/png"),
    (ICON4, icon_file4) => ("../static/img/android-icon-48x48.png", "/android-icon-48x48.png", "image/png"),
    (ICON5, icon_file5) => ("../static/img/android-icon-72x72.png", "/android-icon-72x72.png", "image/png"),
    (ICON6, icon_file6) => ("../static/img/android-icon-96x96.png", "/android-icon-96x96.png", "image/png"),
    (ICON7, icon_file7) => ("../static/img/apple-icon-114x114.png", "/apple-icon-114x114.png", "image/png"),
    (ICON8, icon_file8) => ("../static/img/apple-icon-120x120.png", "/apple-icon-120x120.png", "image/png"),
    (ICON9, icon_file9) => ("../static/img/apple-icon-144x144.png", "/apple-icon-144x144.png", "image/png"),
    (ICON10, icon_file10) => ("../static/img/apple-icon-152x152.png", "/apple-icon-152x152.png", "image/png"),
    (ICON11, icon_file11) => ("../static/img/apple-icon-180x180.png", "/apple-icon-180x180.png", "image/png"),
    (ICON12, icon_file12) => ("../static/img/apple-icon-57x57.png", "/apple-icon-57x57.png", "image/png"),
    (ICON13, icon_file13) => ("../static/img/apple-icon-60x60.png", "/apple-icon-60x60.png", "image/png"),
    (ICON14, icon_file14) => ("../static/img/apple-icon-72x72.png", "/apple-icon-72x72.png", "image/png"),
    (ICON15, icon_file15) => ("../static/img/apple-icon-76x76.png", "/apple-icon-76x76.png", "image/png"),
    (ICON16, icon_file16) => ("../static/img/apple-icon-precomposed.png", "/apple-icon-precomposed.png", "image/png"),
    (ICON17, icon_file17) => ("../static/img/apple-icon.png", "/apple-icon.png", "image/png"),
    (ICON18, icon_file18) => ("../static/img/favicon-16x16.png", "/favicon-16x16.png", "image/png"),
    (ICON19, icon_file19) => ("../static/img/favicon-32x32.png", "/favicon-32x32.png", "image/png"),
    (ICON20, icon_file20) => ("../static/img/favicon-96x96.png", "/favicon-96x96.png", "image/png"),
    (ICON21, icon_file21) => ("../static/img/ms-icon-144x144.png", "/ms-icon-144x144.png", "image/png"),
    (ICON22, icon_file22) => ("../static/img/ms-icon-150x150.png", "/ms-icon-150x150.png", "image/png"),
    (ICON23, icon_file23) => ("../static/img/ms-icon-310x310.png", "/ms-icon-310x310.png", "image/png"),
    (ICON24, icon_file24) => ("../static/img/ms-icon-70x70.png", "/ms-icon-70x70.png", "image/png"),
}

const INDEX_FILE: &[u8] = include_bytes!("../static/index.html");

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .header("Cache-Control", NO_CACHE)
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
