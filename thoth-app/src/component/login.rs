use thoth_api::account::model::AccountDetails;
use thoth_api::account::model::LoginCredentials;
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
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::service::account::AccountError;
use crate::service::account::AccountService;
use crate::string::AUTHENTICATION_ERROR;
use crate::string::INPUT_EMAIL;
use crate::string::INPUT_PASSWORD;
use crate::string::RESPONSE_ERROR;
use crate::string::TEXT_LOGIN;

pub struct LoginComponent {
    request: LoginCredentials,
    response: Callback<Result<AccountDetails, AccountError>>,
    task: Option<FetchTask>,
    link: ComponentLink<Self>,
    account_service: AccountService,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
    props: Props,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub callback: Callback<AccountDetails>,
    pub current_user: Option<AccountDetails>,
}

pub enum Msg {
    RedirectToAdmin,
    Request,
    Response(Result<AccountDetails, AccountError>),
    ChangeEmail(String),
    ChangePassword(String),
}

impl Component for LoginComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        LoginComponent {
            request: Default::default(),
            response: link.callback(Msg::Response),
            task: None,
            link,
            account_service: AccountService::new(),
            notification_bus: NotificationBus::dispatcher(),
            router: RouteAgentDispatcher::new(),
            props,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        // if user is logged in there's no point in seeing the login page
        if first_render && self.props.current_user.is_some() {
            self.link.send_message(Msg::RedirectToAdmin);
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        if self.props.current_user.is_some() {
            self.link.send_message(Msg::RedirectToAdmin);
        }
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RedirectToAdmin => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from(AppRoute::Admin(
                        AdminRoute::Admin,
                    ))));
                false
            }
            Msg::Request => {
                self.task = Some(
                    self.account_service
                        .login(self.request.clone(), self.response.clone()),
                );
                true
            }
            Msg::Response(Ok(account_details)) => {
                let token = account_details.token.clone().unwrap();
                self.account_service.set_token(token);
                self.props.callback.emit(account_details);
                self.task = None;
                self.link.send_message(Msg::RedirectToAdmin);
                true
            }
            Msg::Response(Err(err)) => {
                match err {
                    AccountError::AuthenticationError => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            AUTHENTICATION_ERROR.into(),
                            NotificationStatus::Warning,
                        )));
                    }
                    AccountError::ResponseError => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            RESPONSE_ERROR.into(),
                            NotificationStatus::Danger,
                        )));
                    }
                };
                self.task = None;
                true
            }
            Msg::ChangeEmail(email) => self.request.email.neq_assign(email),
            Msg::ChangePassword(password) => self.request.password.neq_assign(password),
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
                                    value=self.request.email.clone()
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
                                    value=self.request.password.clone()
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
                                    onclick=self.link.callback(|_| Msg::Request)
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
