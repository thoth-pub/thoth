use std::collections::HashSet;
use uuid::Uuid;
use yew::prelude::worker::*;
use yew::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request as NotificationRequest;
use crate::models::funder::funder_activity_query::FetchActionFunderActivity;
use crate::models::funder::funder_activity_query::FetchFunderActivity;
use crate::models::funder::funder_activity_query::FunderActivityRequest;
use crate::models::funder::funder_activity_query::FunderActivityRequestBody;
use crate::models::funder::funder_activity_query::FunderActivityResponseData;
use crate::models::funder::funder_activity_query::Variables;

pub enum Msg {
    SetFunderActivityFetchState(FetchActionFunderActivity),
}

pub enum Request {
    RetrieveFunderActivity(Uuid),
}

pub struct FunderActivityChecker {
    agent_link: AgentLink<FunderActivityChecker>,
    fetch_funder_activity: FetchFunderActivity,
    subscribers: HashSet<HandlerId>,
    notification_bus: NotificationDispatcher,
}

impl Agent for FunderActivityChecker {
    type Input = Request;
    type Message = Msg;
    type Output = FunderActivityResponseData;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            agent_link: link,
            fetch_funder_activity: Default::default(),
            subscribers: HashSet::new(),
            notification_bus: NotificationBus::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::SetFunderActivityFetchState(fetch_state) => {
                self.fetch_funder_activity.apply(fetch_state);
                match self.fetch_funder_activity.as_ref().state() {
                    FetchState::NotFetching(_) => (),
                    FetchState::Fetching(_) => (),
                    FetchState::Fetched(body) => {
                        let response = &body.data;
                        for sub in self.subscribers.iter() {
                            self.agent_link.respond(*sub, response.clone());
                        }
                    }
                    FetchState::Failed(_, err) => {
                        self.notification_bus
                            .send(NotificationRequest::NotificationBusMsg((
                                err.to_string(),
                                NotificationStatus::Danger,
                            )));
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::RetrieveFunderActivity(funder_id) => {
                let body = FunderActivityRequestBody {
                    variables: Variables {
                        funder_id: Some(funder_id),
                    },
                    ..Default::default()
                };
                let request = FunderActivityRequest { body };
                self.fetch_funder_activity = Fetch::new(request);
                self.agent_link.send_future(
                    self.fetch_funder_activity
                        .fetch(Msg::SetFunderActivityFetchState),
                );
                self.agent_link
                    .send_message(Msg::SetFunderActivityFetchState(FetchAction::Fetching));
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
