use std::collections::HashSet;
use uuid::Uuid;
use yew_agent::{Agent, AgentLink, Context, Dispatched, HandlerId};
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request as NotificationRequest;
use crate::models::institution::institution_activity_query::FetchActionInstitutionActivity;
use crate::models::institution::institution_activity_query::FetchInstitutionActivity;
use crate::models::institution::institution_activity_query::InstitutionActivityRequest;
use crate::models::institution::institution_activity_query::InstitutionActivityRequestBody;
use crate::models::institution::institution_activity_query::InstitutionActivityResponseData;
use crate::models::institution::institution_activity_query::Variables;

pub enum Msg {
    SetInstitutionActivityFetchState(FetchActionInstitutionActivity),
}

pub enum Request {
    RetrieveInstitutionActivity(Uuid),
}

pub struct InstitutionActivityChecker {
    agent_link: AgentLink<InstitutionActivityChecker>,
    fetch_institution_activity: FetchInstitutionActivity,
    subscribers: HashSet<HandlerId>,
    notification_bus: NotificationDispatcher,
}

impl Agent for InstitutionActivityChecker {
    type Input = Request;
    type Message = Msg;
    type Output = InstitutionActivityResponseData;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            agent_link: link,
            fetch_institution_activity: Default::default(),
            subscribers: HashSet::new(),
            notification_bus: NotificationBus::dispatcher(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::SetInstitutionActivityFetchState(fetch_state) => {
                self.fetch_institution_activity.apply(fetch_state);
                match self.fetch_institution_activity.as_ref().state() {
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
            Request::RetrieveInstitutionActivity(institution_id) => {
                let body = InstitutionActivityRequestBody {
                    variables: Variables {
                        institution_id: Some(institution_id),
                    },
                    ..Default::default()
                };
                let request = InstitutionActivityRequest { body };
                self.fetch_institution_activity = Fetch::new(request);
                self.agent_link.send_future(
                    self.fetch_institution_activity
                        .fetch(Msg::SetInstitutionActivityFetchState),
                );
                self.agent_link
                    .send_message(Msg::SetInstitutionActivityFetchState(FetchAction::Fetching));
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
