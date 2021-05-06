#[macro_export]
macro_rules! timer_agent {
    (
        $agent:ident,
        $agent_dispatcher:ident,
        $agent_request:ident,
        $agent_response:ident,
    ) => {
        use serde::Deserialize;
        use serde::Serialize;
        use std::time::Duration;
        use yew::agent::Dispatcher;
        use yew::prelude::worker::*;
        use yew::services::IntervalService;
        use yew::services::Task;
        use yew::Callback;

        pub type $agent_dispatcher = Dispatcher<$agent>;

        pub enum $agent_request {
            Start(Callback<()>),
            Stop,
        }

        #[derive(Deserialize, Serialize)]
        pub struct $agent_response;

        pub struct $agent {
            _link: AgentLink<$agent>,
            timer_task: Option<Box<dyn Task>>,
        }

        impl Agent for $agent {
            type Input = $agent_request;
            type Message = ();
            type Output = $agent_response;
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
                    $agent_request::Start(callback) => {
                        let handle = IntervalService::spawn(Duration::from_secs(60), callback);
                        self.timer_task = Some(Box::new(handle));
                    }
                    $agent_request::Stop => {
                        if self.timer_task.take().is_some() {
                            self.timer_task = None;
                        }
                    }
                }
            }
        }
    };
}

pub mod contributor_activity_checker;
pub mod funder_activity_checker;
pub mod notification_bus;
pub mod session_timer;
pub mod version_timer;
