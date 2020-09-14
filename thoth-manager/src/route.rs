use yew_router::prelude::*;
use yew_router::switch::Permissive;

#[derive(Switch, Debug, Clone)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
    #[to="/loading"]
    Loading,
    #[to = "/error"]
    Error(Permissive<String>),
    #[to="/admin/dashboard"]
    Dashboard,
    #[to="/admin/test"]
    Test,
    #[to="/admin"]
    Admin,
    #[to="/"]
    Home,
}
