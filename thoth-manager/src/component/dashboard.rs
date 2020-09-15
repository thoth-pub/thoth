use serde::Deserialize;
use serde::Serialize;
use yew::ComponentLink;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::FetchState;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;
use yewtil::future::LinkFuture;
use yew_router::prelude::RouterAnchor;

use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct DashboardComponent {
    markdown: Fetch<Request, ResponseBody>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchAction<ResponseBody>),
    GetMarkdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    work_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    publisher_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct Request {
    body: RequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseBody {
    data: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    works: Vec<Work>,
    publishers: Vec<Publisher>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestBody {
    query: String,
    variables: String,
}

impl Default for ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            data: ResponseData { works: vec![], publishers: vec![] },
        }
    }
}

impl Default for ResponseData {
    fn default() -> ResponseData {
        ResponseData {
            works: vec![],
            publishers: vec![],
        }
    }
}

impl Default for RequestBody {
    fn default() -> RequestBody {
        RequestBody {
            query: "
                {
                    works(limit: 9999) { workId }
                    publishers(limit: 9999) { publisherId }
                }
            ".to_string(),
            variables: "null".to_string()
        }
    }
}

impl FetchRequest for Request {
    type RequestBody = RequestBody;
    type ResponseBody = ResponseBody;
    type Format = Json;

    fn url(&self) -> String {
        "http://localhost:8000/graphql".to_string()
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![("Content-Type".to_string(), "application/json".to_string())]
    }

    fn use_cors(&self) -> bool {
        true
    }
}

impl Component for DashboardComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DashboardComponent{
            markdown: Default::default(),
            link,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.markdown.fetch(Msg::SetMarkdownFetchState));
            self.link
                .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.markdown.apply(fetch_state);
                true
            }
            Msg::GetMarkdown => {
                self.link
                    .send_future(self.markdown.fetch(Msg::SetMarkdownFetchState));
                self.link
                    .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match self.markdown.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {
                    <div class="buttons has-addons is-centered">
                        <button
                            class="button is-success is-large"
                            onclick=self.link.callback(|_| Msg::GetMarkdown)
                        >
                            <span class="icon">
                            <i class="fas fa-sync"></i>
                            </span>
                            <span>{"Reload"}</span>
                        </button>
                    </div>
                }
            }
            FetchState::Fetching(_) => html! {
                <div class="hero is-medium">
                    <div class="hero-body">
                        <div class="container has-text-centered">
                            <progress class="progress is-warning" max="100"></progress>
                        </div>
                    </div>
                </div>
            },
            FetchState::Fetched(body) => html! {
                <div class="tile is-ancestor">
                    <div class="tile is-parent">
                        <article class="tile is-child notification is-primary">
                            <div class="content">
                                <p class="title">
                                    {format!("{} Works", body.data.works.iter().count())}
                                </p>
                                <RouterAnchor<AppRoute>
                                    route=AppRoute::Admin(AdminRoute::Works)
                                >
                                    {"See all"}
                                </  RouterAnchor<AppRoute>>
                            </div>
                        </article>
                    </div>
                    <div class="tile is-parent">
                        <article class="tile is-child notification is-link">
                            <div class="content">
                                <p class="title">
                                    {format!("{} Publishers", body.data.publishers.iter().count())}
                                </p>
                                <RouterAnchor<AppRoute>
                                    route=AppRoute::Admin(AdminRoute::Publishers)
                                >
                                    {"See all"}
                                </  RouterAnchor<AppRoute>>
                            </div>
                        </article>
                    </div>
                </div>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
