use std::io;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer};

mod manifest;
use crate::manifest::manifest_source;

const NO_CACHE: &str = "no-cache";
const STRICT_TRANSPORT_SECURITY: &str = "max-age=63072000; includeSubDomains; preload";
const X_CONTENT_TYPE_OPTIONS: &str = "nosniff";
const X_FRAME_OPTIONS: &str = "DENY";
const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";
const PERMISSIONS_POLICY: &str = "geolocation=(), camera=(), microphone=()";
const LOG_FORMAT: &str = r#"%{r}a %a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#;

macro_rules! static_files {
    ($(($cname:ident, $fname:ident) => ($source_path:expr, $dest_path:expr, $type:expr),)*) => (
        $(
            const $cname: &[u8] = include_bytes!($source_path);

            #[get($dest_path)]
            async fn $fname() -> HttpResponse {
                HttpResponse::Ok()
                    .content_type($type)
                    .append_header(("Cache-Control", NO_CACHE))
                    .append_header(("Strict-Transport-Security", STRICT_TRANSPORT_SECURITY))
                    .append_header(("X-Content-Type-Options", X_CONTENT_TYPE_OPTIONS))
                    .append_header(("X-Frame-Options", X_FRAME_OPTIONS))
                    .append_header(("Referrer-Policy", REFERRER_POLICY))
                    .append_header(("Permissions-Policy", PERMISSIONS_POLICY))
                    .body($cname)
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
    (CSS, css_file) => ("../static/pkg/thoth.css", "/admin/thoth.css", "text/css; charset=utf-8"),
}

const INDEX_FILE: &[u8] = include_bytes!("../static/pkg/index.html");

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .append_header(("Cache-Control", NO_CACHE))
        .append_header(("Strict-Transport-Security", STRICT_TRANSPORT_SECURITY))
        .append_header(("X-Content-Type-Options", X_CONTENT_TYPE_OPTIONS))
        .append_header(("X-Frame-Options", X_FRAME_OPTIONS))
        .append_header(("Referrer-Policy", REFERRER_POLICY))
        .append_header(("Permissions-Policy", PERMISSIONS_POLICY))
        .body(INDEX_FILE)
}

#[get("/admin/manifest.json")]
async fn app_manifest() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/json")
        .append_header(("Strict-Transport-Security", STRICT_TRANSPORT_SECURITY))
        .append_header(("X-Content-Type-Options", X_CONTENT_TYPE_OPTIONS))
        .append_header(("X-Frame-Options", X_FRAME_OPTIONS))
        .append_header(("Referrer-Policy", REFERRER_POLICY))
        .append_header(("Permissions-Policy", PERMISSIONS_POLICY))
        .body(manifest_source())
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
            .service(app_manifest)
    })
    .workers(threads)
    .keep_alive(Duration::from_secs(keep_alive))
    .bind(format!("{host}:{port}"))?
    .run()
    .await
}
