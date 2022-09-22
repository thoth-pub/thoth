use std::collections::HashSet;
use uuid::Uuid;
use thoth_errors::ThothError;
use yew_agent::{Agent, AgentLink, Context, Dispatched, HandlerId};
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request as NotificationRequest;
use crate::models::contributor::contributor_activity_query::ContributorActivityRequest;
use crate::models::contributor::contributor_activity_query::ContributorActivityRequestBody;
use crate::models::contributor::contributor_activity_query::ContributorActivityResponseData;
use crate::models::contributor::contributor_activity_query::FetchActionContributorActivity;
use crate::models::contributor::contributor_activity_query::FetchContributorActivity;
use crate::models::contributor::contributor_activity_query::Variables;

pub enum Msg {
    SetContributorActivityFetchState(FetchActionContributorActivity),
}

pub enum Request {
    RetrieveContributorActivity(Uuid),
}

pub struct ContributorActivityChecker {
    agent_link: AgentLink<ContributorActivityChecker>,
    fetch_contributor_activity: FetchContributorActivity,
    subscribers: HashSet<HandlerId>,
    notification_bus: NotificationDispatcher,
}

impl Agent for ContributorActivityChecker {
    type Input = Request;
    type Message = Msg;
    type Output = ContributorActivityResponseData;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            agent_link: link,
            fetch_contributor_activity: Default::default(),
            subscribers: HashSet::new(),
            notification_bus: NotificationBus::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::SetContributorActivityFetchState(fetch_state) => {
                self.fetch_contributor_activity.apply(fetch_state);
                match self.fetch_contributor_activity.as_ref().state() {
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
                                ThothError::from(err).to_string(),
                                NotificationStatus::Danger,
                            )));
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::RetrieveContributorActivity(contributor_id) => {
                let body = ContributorActivityRequestBody {
                    variables: Variables {
                        contributor_id: Some(contributor_id),
                    },
                    ..Default::default()
                };
                let request = ContributorActivityRequest { body };
                self.fetch_contributor_activity = Fetch::new(request);
                self.agent_link.send_future(
                    self.fetch_contributor_activity
                        .fetch(Msg::SetContributorActivityFetchState),
                );
                self.agent_link
                    .send_message(Msg::SetContributorActivityFetchState(FetchAction::Fetching));
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
