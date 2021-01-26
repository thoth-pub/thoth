use yew::agent::Dispatcher;
use yew::prelude::worker::*;

pub type ContributorLinksDispatcher = Dispatcher<ContributorLinksAgent>;

pub enum Msg {
    Fetch,
}

//#[derive(Deserialize, Serialize)]
pub enum Request {
    ContributorLinksRequest,
}

//#[derive(Deserialize, Serialize)]
pub struct ContributorLinksResponse;

pub struct ContributorLinksAgent {
    agent_link: AgentLink<ContributorLinksAgent>,
}

impl Agent for ContributorLinksAgent {
    type Input = Request;
    type Message = Msg;
    type Output = ContributorLinksResponse;
    type Reach = Context<Self>;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            agent_link: link,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::Fetch => {
                //todo
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _: HandlerId) {
        match msg {
            Request::ContributorLinksRequest => {
                //todo
            }
        }
    }
}