#[macro_export]
macro_rules! session_timer {
    (
        $agent:ident,
        $agent_dispatcher:ident,
    ) => {
        use serde::Deserialize;
        use serde::Serialize;
        use std::time::Duration;
        use yew::agent::Dispatcher;
        use yew::prelude::worker::*;
        use yew::services::IntervalService;
        use yew::services::Task;

        pub type $agent_dispatcher = Dispatcher<$agent>;

        pub enum TimerRequest {
            Start(Callback<()>),
            Stop,
        }

        #[derive(Deserialize, Serialize)]
        pub struct TimerResponse;

        pub struct $agent {
            _link: AgentLink<$agent>,
            timer_task: Option<Box<dyn Task>>,
        }

        impl Agent for $agent {
            type Input = TimerRequest;
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
                    TimerRequest::Start(callback) => {
                        let handle = IntervalService::spawn(Duration::from_secs(60), callback);
                        self.timer_task = Some(Box::new(handle));
                    }
                    TimerRequest::Stop => {
                        if self.timer_task.take().is_some() {
                            self.timer_task = None;
                        }
                    }
                }
            }
        }
    };
}
