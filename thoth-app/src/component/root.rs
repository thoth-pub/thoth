use semver::Version;
use thoth_api::account::model::AccountDetails;
use thoth_errors::ThothError;
use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew::Callback;
use yew_agent::Dispatched;
use yew_router::prelude::*;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::agent::session_timer::SessionTimerAgent;
use crate::agent::session_timer::SessionTimerDispatcher;
use crate::agent::session_timer::SessionTimerRequest;
use crate::agent::version_timer::VersionTimerAgent;
use crate::agent::version_timer::VersionTimerDispatcher;
use crate::agent::version_timer::VersionTimerRequest;
use crate::component::about::AboutComponent;
use crate::component::admin::AdminComponent;
use crate::component::catalogue::CatalogueComponent;
use crate::component::hero::HeroComponent;
use crate::component::login::LoginComponent;
use crate::component::navbar::NavbarComponent;
use crate::component::notification::NotificationComponent;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::service::account::AccountError;
use crate::service::account::AccountService;
use crate::service::version;
use crate::string::NEW_VERSION_PROMPT;

pub struct RootComponent {
    account_service: AccountService,
    current_user: Option<AccountDetails>,
    session_timer_agent: SessionTimerDispatcher,
    version_timer_agent: VersionTimerDispatcher,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    FetchCurrentUser,
    CurrentUserResponse(Result<AccountDetails, AccountError>),
    RenewToken,
    RenewTokenResponse(Result<AccountDetails, AccountError>),
    CheckVersion,
    CheckVersionResponse(Result<Version, ThothError>),
    UpdateAccount(AccountDetails),
    Login(AccountDetails),
    Logout,
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let session_timer_agent = SessionTimerAgent::dispatcher();
        let version_timer_agent = VersionTimerAgent::dispatcher();
        let notification_bus = NotificationBus::dispatcher();

        RootComponent {
            account_service: AccountService::new(),
            current_user: Default::default(),
            session_timer_agent,
            version_timer_agent,
            notification_bus,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // Start timer to check for updated app version
            self.version_timer_agent.send(VersionTimerRequest::Start(
                ctx.link().callback(|_| Msg::CheckVersion),
            ));
            if self.account_service.is_loggedin() {
                ctx.link().send_message(Msg::FetchCurrentUser);
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchCurrentUser => {
                let mut service = self.account_service.clone();
                ctx.link().send_future(async move {
                    Msg::CurrentUserResponse(service.account_details().await)
                });
            }
            Msg::RenewToken => {
                let mut service = self.account_service.clone();
                ctx.link().send_future(async move {
                    Msg::RenewTokenResponse(service.renew_token().await)
                });
            }
            Msg::CheckVersion => {
                ctx.link()
                    .send_future(async { Msg::CheckVersionResponse(version::get_version().await) });
            }
            Msg::CurrentUserResponse(Ok(account_details)) => {
                ctx.link().send_message(Msg::Login(account_details));
            }
            Msg::CurrentUserResponse(Err(_)) => {
                ctx.link().send_message(Msg::Logout);
            }
            Msg::RenewTokenResponse(Ok(account_details)) => {
                ctx.link().send_message(Msg::UpdateAccount(account_details));
            }
            Msg::RenewTokenResponse(Err(_)) => {
                ctx.link().send_message(Msg::Logout);
            }
            Msg::CheckVersionResponse(Ok(server_version)) => {
                if let Ok(app_version) = Version::parse(env!("CARGO_PKG_VERSION")) {
                    if server_version > app_version {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            NEW_VERSION_PROMPT.into(),
                            NotificationStatus::Success,
                        )));
                        // Don't send repeated notifications.
                        self.version_timer_agent.send(VersionTimerRequest::Stop);
                    }
                }
            }
            Msg::CheckVersionResponse(Err(_)) => {
                // Unable to determine if a new app version is available.
                // Ignore and move on - not worth alerting the user.
            }
            Msg::UpdateAccount(account_details) => {
                self.current_user = Some(account_details);
            }
            Msg::Login(account_details) => {
                // start session timer
                self.session_timer_agent.send(SessionTimerRequest::Start(
                    ctx.link().callback(|_| Msg::RenewToken),
                ));
                ctx.link().send_message(Msg::UpdateAccount(account_details));
            }
            Msg::Logout => {
                self.account_service.logout();
                self.session_timer_agent.send(SessionTimerRequest::Stop);
                self.current_user = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> VNode {
        let callback_login = ctx.link().callback(Msg::Login);
        let callback_logout = ctx.link().callback(|_| Msg::Logout);
        let current_user = self.current_user.clone();
        let render =
            Switch::render(move |r| switch_app(r, current_user.clone(), callback_login.clone()));

        html! {
            <BrowserRouter>
                <header>
                    <NavbarComponent current_user={ self.current_user.clone() } callback={ callback_logout }/>
                </header>
                <NotificationComponent />
                <div class="main">
                    <Switch<AppRoute> { render } />
                </div>
            </BrowserRouter>
        }
    }
}

fn switch_app(
    route: &AppRoute,
    current_user: Option<AccountDetails>,
    callback_login: Callback<AccountDetails>,
) -> Html {
    match route {
        AppRoute::Home => html! {
            <>
                <HeroComponent />
                <div class="section">
                    <CatalogueComponent />
                </div>
            </>
        },
        AppRoute::About => html! {
            <div class="section">
                <AboutComponent />
            </div>
        },
        AppRoute::Login => html! {
            <div class="section">
                <LoginComponent current_user={ current_user.clone() } callback={ callback_login }/>
            </div>
        },
        AppRoute::Admin => html! {
            <div class="section">
                <AdminComponent current_user={ current_user.clone() }/>
            </div>
        },
        AppRoute::AdminHome => html! {
            <Redirect<AdminRoute> to={ AdminRoute::Dashboard }/>
        },
        AppRoute::Error => html! {
            "Page not found"
        },
    }
}
