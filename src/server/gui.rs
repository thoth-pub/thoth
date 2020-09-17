use std::io;

use actix_cors::Cors;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::web;
use dotenv::dotenv;

const INDEX_FILE: &[u8] = include_bytes!("../static/index.html");
const MATOMO_FILE: &[u8] = include_bytes!("../static/js/matomo.js");
const JS_FILE: &[u8] = include_bytes!("../static/main.js");
const PKG_FILE: &[u8] = include_bytes!("../static/pkg/thoth_manager.js");
const WASM_FILE: &[u8] = include_bytes!("../static/pkg/thoth_manager_bg.wasm");
const BULMA_FILE: &[u8] = include_bytes!("../static/css/bulma-pageloader.min.css");
const CSS_FILE: &[u8] = include_bytes!("../static/css/thoth.css");
const ICON_FILE: &[u8] = include_bytes!("../static/favicon.ico");
const LOGO_FILE: &[u8] = include_bytes!("../static/img/thoth-logo.png");

#[get("/pkg/thoth_manager_bg.wasm")]
async fn wasm_file() -> HttpResponse {
    HttpResponse::Ok().content_type("application/wasm").body(WASM_FILE)
}

#[get("/main.js")]
async fn js_file() -> HttpResponse {
    HttpResponse::Ok().content_type("application/javascript").body(JS_FILE)
}

#[get("/js/matomo.js")]
async fn matomo_file() -> HttpResponse {
    HttpResponse::Ok().content_type("application/javascript").body(MATOMO_FILE)
}

#[get("/pkg/thoth_manager.js")]
async fn pkg_file() -> HttpResponse {
    HttpResponse::Ok().content_type("application/javascript").body(PKG_FILE)
}

#[get("/css/thoth.css")]
async fn css_file() -> HttpResponse {
    HttpResponse::Ok().content_type("text/css; charset=utf-8").body(CSS_FILE)
}

#[get("/css/bulma-pageloader.min.css")]
async fn bulma_file() -> HttpResponse {
    HttpResponse::Ok().content_type("text/css; charset=utf-8").body(BULMA_FILE)
}

#[get("/favicon.ico")]
async fn favicon() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("image/x-icon")
        .body(ICON_FILE)
}

#[get("/img/thoth-logo.png")]
async fn logo() -> HttpResponse {
    HttpResponse::Ok().content_type("image/png").body(LOGO_FILE)
}


async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_FILE)
}

fn config(cfg: &mut web::ServiceConfig) {
    dotenv().ok();
    cfg.service(wasm_file);
    cfg.service(js_file);
    cfg.service(matomo_file);
    cfg.service(pkg_file);
    cfg.service(css_file);
    cfg.service(bulma_file);
    cfg.service(favicon);
    cfg.service(logo);
}

#[actix_rt::main]
pub async fn start_server(port: String) -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .finish()
            )
            .configure(config)
            .default_service(web::route().to(index))
        })
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}
