use thoth_api::account::model::Login;
use thoth_api::account::model::LoginCredentials;
use thoth_api::account::model::Session;
use yew::format::Json;
use yew::html;
use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::fetch;
use crate::models::Response;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::service::cookie::CookieService;
use crate::string::AUTHENTICATION_ERROR;
use crate::string::INPUT_EMAIL;
use crate::string::INPUT_PASSWORD;
use crate::string::RESPONSE_ERROR;
use crate::string::TEXT_LOGIN;
use crate::SESSION_COOKIE;

pub struct LoginComponent {
    email: String,
    password: String,
    fetch_task: Option<FetchTask>,
    link: ComponentLink<Self>,
    cookie_service: CookieService,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    LoginRequest,
    Fetch(Response<Login>),
    ChangeEmail(String),
    ChangePassword(String),
}

impl Component for LoginComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let email = "".into();
        let password = "".into();
        let cookie_service = CookieService::new();
        let notification_bus = NotificationBus::dispatcher();
        let router = RouteAgentDispatcher::new();

        LoginComponent {
            email,
            password,
            fetch_task: None,
            link,
            cookie_service,
            notification_bus,
            router,
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::LoginRequest => {
                self.fetch_task = fetch! {
                    LoginCredentials {
                        email: self.email.to_owned(),
                        password: self.password.to_owned(),
                    } => "/account/login",
                    self.link, Msg::Fetch,
                    || {},
                    || {
                        log::error!("Unable to create login request");
                        self.notification_bus.send(Request::NotificationBusMsg((
                            RESPONSE_ERROR.into(),
                            NotificationStatus::Danger,
                        )));
                    }
                };
                false
            }
            Msg::Fetch(response) => {
                let (meta, Json(body)) = response.into_parts();

                if meta.status.is_success() {
                    match body {
                        Ok(Login(Session { token })) => {
                            self.cookie_service.set(SESSION_COOKIE, &token);
                            self.router.send(RouteRequest::ChangeRoute(Route::from(
                                AppRoute::Admin(AdminRoute::Admin),
                            )));
                        }
                        _ => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                RESPONSE_ERROR.into(),
                                NotificationStatus::Danger,
                            )));
                        }
                    }
                } else {
                    self.notification_bus.send(Request::NotificationBusMsg((
                        AUTHENTICATION_ERROR.into(),
                        NotificationStatus::Warning,
                    )));
                }
                self.fetch_task = None;
                true
            }
            Msg::ChangeEmail(email) => self.email.neq_assign(email),
            Msg::ChangePassword(password) => self.password.neq_assign(password),
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="columns is-mobile is-centered">
                <div class="column is-3">
                    <div class="box">
                        <div class="field">
                            <p class="control has-icons-left has-icons-right">
                                <input
                                    class="input"
                                    type="email"
                                    value=&self.email
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeEmail(e.value))
                                    placeholder=INPUT_EMAIL
                                />
                                <span class="icon is-small is-left">
                                    <i class="fas fa-envelope"></i>
                                </span>
                            </p>
                        </div>
                        <div class="field">
                            <p class="control has-icons-left">
                                <input
                                    class="input"
                                    type="password"
                                    value=&self.password
                                    oninput=self.link.callback(|e: InputData| Msg::ChangePassword(e.value))
                                    placeholder=INPUT_PASSWORD
                                />
                                <span class="icon is-small is-left">
                                    <i class="fas fa-lock"></i>
                                </span>
                            </p>
                        </div>
                        <div class="field">
                            <p class="control">
                                <button
                                    class="button is-success"
                                    onclick=self.link.callback(|_| Msg::LoginRequest)
                                >
                                    { TEXT_LOGIN }
                                </button>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
