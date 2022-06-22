#[macro_export]
macro_rules! timer_agent {
    (
        $agent:ident,
        $agent_dispatcher:ident,
        $agent_request:ident,
        $agent_response:ident,
    ) => {
        use gloo_timers::callback::Interval;
        use serde::Deserialize;
        use serde::Serialize;
        use yew::Callback;
        use yew_agent::{Agent, AgentLink, Context, Dispatcher, HandlerId};

        pub type $agent_dispatcher = Dispatcher<$agent>;

        pub enum $agent_request {
            Start(Callback<()>),
            Stop,
        }

        #[derive(Deserialize, Serialize)]
        pub struct $agent_response;

        pub struct $agent {
            _link: AgentLink<$agent>,
            timer_task: Option<Interval>,
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
                        self.timer_task = Some(Interval::new(60_000, move || callback.emit(())));
                    }
                    $agent_request::Stop => {
                        if let Some(timer_task) = self.timer_task.take() {
                            // .take() sets self.timer_task to None so no need to do it explicitly
                            timer_task.cancel();
                        }
                    }
                }
            }
        }
    };
}

pub mod contributor_activity_checker;
pub mod institution_activity_checker;
pub mod notification_bus;
pub mod session_timer;
pub mod version_timer;
