use std::io;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer};

const NO_CACHE: &str = "no-cache";
const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#;

macro_rules! static_files {
    ($(($cname:ident, $fname:ident) => ($source_path:expr, $dest_path:expr, $type:expr),)*) => (
        $(
            const $cname: &[u8] = include_bytes!($source_path);

            #[get($dest_path)]
            async fn $fname() -> HttpResponse {
                HttpResponse::Ok().content_type($type).append_header(("Cache-Control", NO_CACHE)).body($cname)
            }
        )*

        fn config(cfg: &mut web::ServiceConfig) {
            $(cfg.service($fname);)*
        }

    )
}

static_files! {
    (JS, js_file) => ("../static/pkg/thoth-app.js", "/admin/thoth-app.js", "application/javascript"),
    (WASM, wasm_file) => ("../static/pkg/thoth-app_bg.wasm", "/admin/thoth-app_bg.wasm", "application/wasm"),
    (BULMA, bulma_file) => ("../static/pkg/bulma-pageloader.min.css", "/admin/bulma-pageloader.min.css", "text/css; charset=utf-8"),
    (CSS, css_file) => ("../static/pkg/thoth.css", "/admin/thoth.css", "text/css; charset=utf-8"),
    (LOGO, logo_file) => ("../static/img/thoth-logo.png", "/admin/img/thoth-logo.png", "image/png"),
    (COVER, cover_file) => ("../static/img/cover-placeholder.jpg", "/admin/img/cover-placeholder.jpg", "image/jpg"),
    (XML, xml_file) => ("../static/browserconfig.xml", "/admin/browserconfig.xml", "application/xml"),
    (JSON, json_file) => ("../static/manifest.json", "/admin/manifest.json", "application/json"),
    (ICON, icon_file) => ("../static/img/favicon.ico", "/admin/favicon.ico", "image/x-icon"),
    (ICON1, icon_file1) => ("../static/img/android-icon-144x144.png", "/admin/android-icon-144x144.png", "image/png"),
    (ICON2, icon_file2) => ("../static/img/android-icon-192x192.png", "/admin/android-icon-192x192.png", "image/png"),
    (ICON3, icon_file3) => ("../static/img/android-icon-36x36.png", "/admin/android-icon-36x36.png", "image/png"),
    (ICON4, icon_file4) => ("../static/img/android-icon-48x48.png", "/admin/android-icon-48x48.png", "image/png"),
    (ICON5, icon_file5) => ("../static/img/android-icon-72x72.png", "/admin/android-icon-72x72.png", "image/png"),
    (ICON6, icon_file6) => ("../static/img/android-icon-96x96.png", "/admin/android-icon-96x96.png", "image/png"),
    (ICON7, icon_file7) => ("../static/img/apple-icon-114x114.png", "/admin/apple-icon-114x114.png", "image/png"),
    (ICON8, icon_file8) => ("../static/img/apple-icon-120x120.png", "/admin/apple-icon-120x120.png", "image/png"),
    (ICON9, icon_file9) => ("../static/img/apple-icon-144x144.png", "/admin/apple-icon-144x144.png", "image/png"),
    (ICON10, icon_file10) => ("../static/img/apple-icon-152x152.png", "/admin/apple-icon-152x152.png", "image/png"),
    (ICON11, icon_file11) => ("../static/img/apple-icon-180x180.png", "/admin/apple-icon-180x180.png", "image/png"),
    (ICON12, icon_file12) => ("../static/img/apple-icon-57x57.png", "/admin/apple-icon-57x57.png", "image/png"),
    (ICON13, icon_file13) => ("../static/img/apple-icon-60x60.png", "/admin/apple-icon-60x60.png", "image/png"),
    (ICON14, icon_file14) => ("../static/img/apple-icon-72x72.png", "/admin/apple-icon-72x72.png", "image/png"),
    (ICON15, icon_file15) => ("../static/img/apple-icon-76x76.png", "/admin/apple-icon-76x76.png", "image/png"),
    (ICON16, icon_file16) => ("../static/img/apple-icon-precomposed.png", "/admin/apple-icon-precomposed.png", "image/png"),
    (ICON17, icon_file17) => ("../static/img/apple-icon.png", "/admin/apple-icon.png", "image/png"),
    (ICON18, icon_file18) => ("../static/img/favicon-16x16.png", "/admin/favicon-16x16.png", "image/png"),
    (ICON19, icon_file19) => ("../static/img/favicon-32x32.png", "/admin/favicon-32x32.png", "image/png"),
    (ICON20, icon_file20) => ("../static/img/favicon-96x96.png", "/admin/favicon-96x96.png", "image/png"),
    (ICON21, icon_file21) => ("../static/img/ms-icon-144x144.png", "/admin/ms-icon-144x144.png", "image/png"),
    (ICON22, icon_file22) => ("../static/img/ms-icon-150x150.png", "/admin/ms-icon-150x150.png", "image/png"),
    (ICON23, icon_file23) => ("../static/img/ms-icon-310x310.png", "/admin/ms-icon-310x310.png", "image/png"),
    (ICON24, icon_file24) => ("../static/img/ms-icon-70x70.png", "/admin/ms-icon-70x70.png", "image/png"),
}

const INDEX_FILE: &[u8] = include_bytes!("../static/pkg/index.html");

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .append_header(("Cache-Control", NO_CACHE))
        .body(INDEX_FILE)
}

#[actix_web::main]
pub async fn start_server(
    host: String,
    port: String,
    threads: usize,
    keep_alive: u64,
) -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(LOG_FORMAT))
            .wrap(Cors::default().allowed_methods(vec!["GET", "POST", "OPTIONS"]))
            .configure(config)
            .default_service(web::route().to(index))
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
