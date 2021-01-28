use std::collections::HashSet;
use yew::prelude::worker::*;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::agent::contributor_links_query::FetchActionContributorLinks;
use crate::agent::contributor_links_query::FetchContributorLinks;
use crate::agent::contributor_links_query::ContributorLink;
use crate::agent::contributor_links_query::ContributorLinksRequest;
use crate::agent::contributor_links_query::ContributorLinksRequestBody;
use crate::agent::contributor_links_query::Variables;

pub enum Msg {
    SetContributorLinksFetchState(FetchActionContributorLinks),
}

//#[derive(Deserialize, Serialize)]
pub enum Request {
    RetrieveContributorLinks(String),
}

//#[derive(Deserialize, Serialize)]
#[derive(Clone)]
pub struct ContributorLinksResponse {
    pub contributor_link: ContributorLink,
}

pub struct ContributorLinksAgent {
    agent_link: AgentLink<ContributorLinksAgent>,
    fetch_contributor_links: FetchContributorLinks,
    subscribers: HashSet<HandlerId>,
}

impl Agent for ContributorLinksAgent {
    type Input = Request;
    type Message = Msg;
    type Output = ContributorLinksResponse;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            agent_link: link,
            fetch_contributor_links: Default::default(),
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::SetContributorLinksFetchState(fetch_state) => {
                self.fetch_contributor_links.apply(fetch_state);
                match self.fetch_contributor_links.as_ref().state() {
                    FetchState::NotFetching(_) => (), //todo
                    FetchState::Fetching(_) => (), //todo
                    FetchState::Fetched(body) => {
                        let contributor_link = match &body.data.contributor {
                            Some(c) => c.to_owned(),
                            None => Default::default(),
                        };
                        let response = ContributorLinksResponse { contributor_link };
                        for sub in self.subscribers.iter() {
                            self.agent_link.respond(*sub, response.clone());
                        }
                    }
                    FetchState::Failed(_, _err) => (), //todo
                }
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::RetrieveContributorLinks(contributor_id) => {
                let body = ContributorLinksRequestBody {
                    variables: Variables {
                        contributor_id: Some(contributor_id),
                    },
                    ..Default::default()
                };
                let request = ContributorLinksRequest { body };
                self.fetch_contributor_links = Fetch::new(request);
                self.agent_link
                    .send_future(self.fetch_contributor_links.fetch(Msg::SetContributorLinksFetchState));
                self.agent_link
                    .send_message(Msg::SetContributorLinksFetchState(FetchAction::Fetching));
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