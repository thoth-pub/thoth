use yew::Callback;
use yew::html;
use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;
use yew_router::route::Route;
use yew_router::switch::Permissive;
use thoth_api::account::model::AccountDetails;

use crate::agent::session_timer;
use crate::agent::session_timer::SessionTimerAgent;
use crate::component::admin::AdminComponent;
use crate::component::catalogue::CatalogueComponent;
use crate::component::hero::HeroComponent;
use crate::component::login::LoginComponent;
use crate::component::navbar::NavbarComponent;
use crate::component::notification::NotificationComponent;
use crate::route::AppRoute;
use crate::service::account::AccountService;
use crate::service::account::AccountError;

pub struct RootComponent {
    current_route: Option<AppRoute>,
    account_service: AccountService,
    current_user: Option<AccountDetails>,
    current_user_response: Callback<Result<AccountDetails, AccountError>>,
    current_user_task: Option<FetchTask>,
    #[allow(unused)]
    router_agent: Box<dyn Bridge<RouteAgent>>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    FetchCurrentUser,
    CurrentUserResponse(Result<AccountDetails, AccountError>),
    Route(Route),
    Logout,
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router_agent = RouteAgent::bridge(link.callback(Msg::Route));
        let route_service: RouteService = RouteService::new();
        let route = route_service.get_route();

        RootComponent {
            current_route: AppRoute::switch(route),
            account_service: AccountService::new(),
            current_user: Default::default(),
            current_user_response: link.callback(Msg::CurrentUserResponse),
            current_user_task: Default::default(),
            router_agent,
            link,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render && self.account_service.is_loggedin() {
            self.link.send_message(Msg::FetchCurrentUser);
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchCurrentUser => {
                let task = self.account_service.account_details(self.current_user_response.clone());
                self.current_user_task = Some(task);
            }
            Msg::CurrentUserResponse(Ok(account_details)) => {
                let mut session_timer_agent = SessionTimerAgent::dispatcher();
                session_timer_agent.send(session_timer::Request::Start);
                self.current_user = Some(account_details);
                self.current_user_task = None;
            }
            Msg::CurrentUserResponse(Err(_)) => {
                self.current_user_task = None;
            }
            Msg::Route(route) => {
                self.current_route = AppRoute::switch(route)
            }
            Msg::Logout => {
                self.account_service.logout();
                self.current_user = None;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> VNode {
        let callback_login = self.link.callback(|_| Msg::FetchCurrentUser);
        let callback_logout = self.link.callback(|_| Msg::Logout);

        html! {
            <>
                <header>
                    <NavbarComponent current_user=&self.current_user callback=callback_logout/>
                </header>
                <NotificationComponent />
                <div class="main">
                {
                    if let Some(route) = &self.current_route {
                        match route {
                            AppRoute::Home => html! {
                                <>
                                    <HeroComponent />
                                    <div class="section">
                                        <CatalogueComponent />
                                    </div>
                                </>
                            },
                            AppRoute::Login => html! {
                                <div class="section">
                                    <LoginComponent current_user=&self.current_user callback=callback_login/>
                                </div>
                            },
                            AppRoute::Admin(admin_route) => html! {
                                <div class="section">
                                    <AdminComponent route={admin_route} current_user=&self.current_user/>
                                </div>
                            },
                            AppRoute::Error(Permissive(None)) => html! {
                                <div class="uk-position-center"></div>
                            },
                            AppRoute::Error(Permissive(Some(missed_route))) => html!{
                                format!("Page '{}' not found", missed_route)
                            }
                        }
                    } else {
                        html! {}
                    }
                }
                </div>
            </>
        }
    }
}
