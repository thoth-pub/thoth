use yew::prelude::worker::*;

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

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::ContributorLinksRequest => {
                self.agent_link.respond(id, ContributorLinksResponse);
            }
        }
    }
}