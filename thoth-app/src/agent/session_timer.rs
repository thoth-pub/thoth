use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use thoth_api::account::model::Login;
use thoth_api::account::model::LoginSession;
use thoth_api::account::model::Session;
use yew::agent::Dispatcher;
use yew::format::Json;
use yew::prelude::worker::*;
use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::services::IntervalService;
use yew::services::Task;

use crate::authenticated_fetch;
use crate::models::Response;
use crate::service::account::AccountService;

pub type SessionTimerDispatcher = Dispatcher<SessionTimerAgent>;

pub enum Msg {
    Fetch(Response<Login>),
    Update,
}

#[derive(Deserialize, Serialize)]
pub enum Request {
    Start,
    Stop,
}

#[derive(Deserialize, Serialize)]
pub struct TimerResponse;

pub struct SessionTimerAgent {
    agent_link: AgentLink<SessionTimerAgent>,
    callback: Callback<()>,
    account_service: AccountService,
    fetch_task: Option<FetchTask>,
    timer_task: Option<Box<dyn Task>>,
}

impl Agent for SessionTimerAgent {
    type Input = Request;
    type Message = Msg;
    type Output = TimerResponse;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            callback: link.callback(|_| Msg::Update),
            agent_link: link,
            account_service: AccountService::new(),
            fetch_task: None,
            timer_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Update => {
                log::info!("Updating current session");
                if let Some(token) = self.account_service.get_token() {
                    self.fetch_task = authenticated_fetch! {
                        LoginSession(Session::new(token)) => "/account/token/renew",
                        token,
                        self.agent_link, Msg::Fetch,
                        || {},
                        || {
                            log::warn!("Unable to create scheduled session login request");
                        }
                    };
                }
            }
            Msg::Fetch(response) => {
                let (meta, Json(body)) = response.into_parts();

                // Check the response type
                if meta.status.is_success() {
                    match body {
                        Ok(Login(Session { token })) => {
                            log::info!("Scheduled session based login succeed");

                            // Set the retrieved session cookie
                            self.account_service.set_token(token);
                        }
                        _ => log::warn!("Got wrong scheduled session login response"),
                    }
                } else {
                    // Authentication failed
                    log::info!(
                        "Scheduled session login failed with status: {}",
                        meta.status
                    );
                }

                // Remove the ongoing task
                self.fetch_task = None;
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::Start => {
                let handle = IntervalService::spawn(Duration::from_secs(60), self.callback.clone());
                self.timer_task = Some(Box::new(handle));
            }
            Request::Stop => {
                if self.timer_task.take().is_some() {
                    self.timer_task = None;
                }
            }
        }
    }
}
