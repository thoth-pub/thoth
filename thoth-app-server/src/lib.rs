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
    (JSON, json_file) => ("../static/manifest.json", "/admin/manifest.json", "application/json"),
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
