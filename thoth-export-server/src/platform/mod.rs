mod handler;
pub(crate) mod model;

use self::handler::{get_all, get_one};
use paperclip::actix::web;

pub(crate) fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/platforms")
            .service(web::resource("/").route(web::get().to(get_all)))
            .service(web::resource("/{platform_id}").route(web::get().to(get_one))),
    );
}
