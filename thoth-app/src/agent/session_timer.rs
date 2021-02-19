use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;
use yew::agent::Dispatcher;
use yew::prelude::worker::*;
use yew::prelude::*;
use yew::services::IntervalService;
use yew::services::Task;

pub type SessionTimerDispatcher = Dispatcher<SessionTimerAgent>;

pub enum Request {
    Start(Callback<()>),
    Stop,
}

#[derive(Deserialize, Serialize)]
pub struct TimerResponse;

pub struct SessionTimerAgent {
    _link: AgentLink<SessionTimerAgent>,
    timer_task: Option<Box<dyn Task>>,
}

impl Agent for SessionTimerAgent {
    type Input = Request;
    type Message = ();
    type Output = TimerResponse;
    type Reach = Context<Self>;

    fn create(_link: AgentLink<Self>) -> Self {
        Self {
            _link,
            timer_task: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::Start(callback) => {
                let handle = IntervalService::spawn(Duration::from_secs(60), callback.clone());
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
