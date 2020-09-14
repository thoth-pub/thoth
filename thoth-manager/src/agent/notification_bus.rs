use std::collections::HashSet;
use std::fmt;
use yew::agent::Dispatcher;
use yew::worker::*;

pub type NotificationDispatcher = Dispatcher<NotificationBus>;

#[derive(Debug)]
pub enum Request {
    NotificationBusMsg((String, NotificationStatus)),
}

#[derive(Debug, Clone)]
pub enum NotificationStatus {
    Primary,
    Link,
    Info,
    Success,
    Warning,
    Danger,
}

impl fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationStatus::Primary => write!(f, "is-primary"),
            NotificationStatus::Link => write!(f, "is-link"),
            NotificationStatus::Info => write!(f, "is-info"),
            NotificationStatus::Success => write!(f, "is-success"),
            NotificationStatus::Warning => write!(f, "is-warning"),
            NotificationStatus::Danger => write!(f, "is-danger"),
        }
    }
}

pub struct NotificationBus {
    link: AgentLink<NotificationBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for NotificationBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = (String, NotificationStatus);

    fn create(link: AgentLink<Self>) -> Self {
        NotificationBus {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::NotificationBusMsg(s) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, s.clone());
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
