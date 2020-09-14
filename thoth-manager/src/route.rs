use yew_router::prelude::*;
use yew_router::switch::Permissive;

#[derive(Switch, Debug, Clone)]
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

#[derive(Switch, Debug, Clone)]
pub enum AdminRoute {
    #[to = "/dashboard"]
    Dashboard,
    #[to = "/test"]
    Test,
    #[to = ""]
    Admin,
}
