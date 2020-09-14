use yew_router::prelude::*;
use yew_router::switch::Permissive;

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
    #[to = "/error"]
    Error(Permissive<String>),
    #[to = "/admin{*:rest}"]
    Admin(AdminRoute),
    #[to = "/"]
    Home,
}

#[derive(Switch, Debug, Clone, PartialEq, Copy)]
pub enum AdminRoute {
    #[to = "/dashboard"]
    Dashboard,
    #[to = "/works"]
    Works,
    #[to = ""]
    Admin,
}
