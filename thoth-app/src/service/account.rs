use yew::services::storage::{Area, StorageService};
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;

use crate::route::AppRoute;
use crate::SESSION_KEY;
use crate::string::STORAGE_ERROR;

pub struct AccountService {
    login_route: Route,
}

impl AccountService {
    pub fn new() -> Self {
        let login_route = Route::from(AppRoute::Login);
        Self { login_route }
    }

    pub fn get_token(&self) -> Option<String> {
        let storage_service = StorageService::new(Area::Local).expect(STORAGE_ERROR);
        if let Ok(token) = storage_service.restore(SESSION_KEY) {
            log::debug!("Get token: {}", token);
            Some(token)
        } else {
            None
        }
    }

    pub fn set_token(&self, token: String) {
        log::debug!("Set token: {}", token);
        self.update_storage(Some(token))
    }

    fn update_storage(&self, token: Option<String>) {
        let mut storage_service = StorageService::new(Area::Local).expect(STORAGE_ERROR);
        if let Some(t) = token {
            storage_service.store(SESSION_KEY, Ok(t));
        } else {
            storage_service.remove(SESSION_KEY);
        }
    }

    pub fn is_loggedin(&self) -> bool {
        let is_loggedin = self.get_token().is_some();
        log::debug!("is_loggedin: {}", is_loggedin);
        is_loggedin
    }

    pub fn logout(&self) {
        log::debug!("Logout");
        self.update_storage(None)
    }

    pub fn redirect_to_login(&self) {
        let mut router: RouteAgentDispatcher<()> = RouteAgentDispatcher::new();
        router.send(RouteRequest::ChangeRoute(self.login_route.clone()))
    }
}
