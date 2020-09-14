use crate::{
    api::Response,
    route::AppRoute,
    service::{
        cookie::CookieService,
        session_timer::{self, SessionTimerAgent},
    },
    string::{REQUEST_ERROR, RESPONSE_ERROR},
    SESSION_COOKIE,
};
use log::{error, info, warn};
use thoth_api::{
    models::account::Session,
    request,
    response,
    API_URL_LOGOUT,
};
use yew::{agent::Bridged, format::Json, html, prelude::*, services::fetch::FetchTask};
use yew_router::agent::{RouteAgent, RouteRequest::ChangeRoute};

pub struct DashboardComponent {
    component_link: ComponentLink<DashboardComponent>,
    cookie_service: CookieService,
    fetch_task: Option<FetchTask>,
    logout_button_disabled: bool,
    router_agent: Box<dyn Bridge<RouteAgent<()>>>,
    session_timer_agent: Box<dyn Bridge<SessionTimerAgent>>,
}

pub enum Message {
    Fetch(Response<response::Logout>),
    Ignore,
    LogoutRequest,
}

impl Component for DashboardComponent {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Guard the authentication
        let mut router_agent = RouteAgent::bridge(link.callback(|_| Message::Ignore));
        let cookie_service = CookieService::new();
        let mut session_timer_agent = SessionTimerAgent::bridge(link.callback(|_| Message::Ignore));
        if cookie_service.get(SESSION_COOKIE).is_err() {
            info!("No session token found, routing back to login");
        } else {
            // Start the timer to keep the session active
            session_timer_agent.send(session_timer::Request::Start);
        }

        // Return the component
        Self {
            component_link: link,
            cookie_service,
            fetch_task: None,
            logout_button_disabled: false,
            router_agent,
            session_timer_agent,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Message::LogoutRequest => {
                if let Ok(token) = self.cookie_service.get(SESSION_COOKIE) {
                    self.fetch_task = fetch! {
                        request::Logout(Session::new(token)) => API_URL_LOGOUT,
                        self.component_link, Message::Fetch,
                        || {
                            // Disable user interaction
                            self.logout_button_disabled = true;
                        },
                        || {
                            error!("Unable to create logout request");
                        }
                    };
                } else {
                    // It should not happen but in case there is no session cookie on logout, route
                    // back to login
                    error!("No session cookie found");
                }
            }

            // The message for all fetch responses
            Message::Fetch(response) => {
                let (meta, Json(body)) = response.into_parts();

                // Check the response type
                if meta.status.is_success() {
                    match body {
                        Ok(response::Logout) => info!("Got valid logout response"),
                        _ => {
                            warn!("Got wrong logout response");
                        }
                    }
                } else {
                    warn!("Logout failed with status: {}", meta.status);
                }

                // Remove the existing cookie
                self.cookie_service.remove(SESSION_COOKIE);
                self.session_timer_agent.send(session_timer::Request::Stop);
                self.logout_button_disabled = true;

                // Remove the ongoing task
                self.fetch_task = None;
            }
            Message::Ignore => {}
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
        <>
            <section class="header"> {"Admin Dashboard"} </section>
        </>
        }
    }
}
