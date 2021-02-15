use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;

use crate::SESSION_COOKIE;
use crate::route::AppRoute;
use crate::service::cookie::CookieService;

pub struct AccountService {
    cookie_service: CookieService,
    login_route: Route,
}

impl AccountService {
    pub fn new() -> Self {
        let cookie_service = CookieService::new();
        let login_route = Route::from(AppRoute::Login);
        Self {
            cookie_service,
            login_route,
        }
    }

    pub fn get_token(&self) -> Option<String> {
        log::debug!("Get token: {}", self.cookie_service.get(SESSION_COOKIE).ok().unwrap_or("".to_string()));
        self.cookie_service.get(SESSION_COOKIE).ok()
    }

    pub fn set_token(&self, token: &str) {
        log::debug!("Set token: {}", token);
        self.cookie_service.set(SESSION_COOKIE, token)
    }

    pub fn is_loggedin(&self) -> bool {
        log::debug!("is_loggedin: {}", !self.cookie_service.get(SESSION_COOKIE).is_err());
        !self.cookie_service.get(SESSION_COOKIE).is_err()
    }

    pub fn logout(&self) {
        log::debug!("Logout");
        self.cookie_service.delete(SESSION_COOKIE)
    }

    pub fn redirect_to_login(&self) {
        let mut router: RouteAgentDispatcher<()> = RouteAgentDispatcher::new();
        router.send(RouteRequest::ChangeRoute(self.login_route.clone()))
    }
}
