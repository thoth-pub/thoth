mod handler;
pub(crate) mod model;

use self::handler::{by_work, get_all, get_one};
use paperclip::actix::web;

pub(crate) fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/specifications")
            .service(web::resource("/").route(web::get().to(get_all)))
            .service(web::resource("/{specification_id}").route(web::get().to(get_one)))
            .service(
                web::resource("/{specification_id}/work/{work_id}").route(web::get().to(by_work)),
            ),
    );
}
