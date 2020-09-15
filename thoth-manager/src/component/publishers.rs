use serde::Deserialize;
use serde::Serialize;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchRequest;
use yewtil::fetch::FetchState;
use yewtil::fetch::Json;
use yewtil::fetch::MethodBody;
use yewtil::future::LinkFuture;
use yew_router::route::Route;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;

use crate::component::utils::Loader;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct PublishersComponent {
    markdown: Fetch<Request, ResponseBody>,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchAction<ResponseBody>),
    GetMarkdown,
    ChangeRoute(AppRoute),
}

#[derive(Default, Debug, Clone)]
pub struct Request {
    body: RequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    publisher_id: String,
    publisher_name: String,
    publisher_shortname: Option<String>,
    publisher_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseBody {
    data: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
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
            data: ResponseData { publishers: vec![] },
        }
    }
}

impl Default for ResponseData {
    fn default() -> ResponseData {
        ResponseData {
            publishers: vec![],
        }
    }
}

impl Default for RequestBody {
    fn default() -> RequestBody {
        RequestBody {
            query: "
                {
                    publishers(limit: 9999) {
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                    }
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

impl Component for PublishersComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();

        PublishersComponent {
            markdown: Default::default(),
            link,
            router,
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
            Msg::ChangeRoute(r) => {
                let route = Route::from(r.clone());
                self.router.send(RouteRequest::ChangeRoute(route));
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
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => html! {
                <>
                    <nav class="level">
                        <div class="level-left">
                            <div class="level-item">
                                <p class="subtitle is-5">
                                    <strong>{ body.data.publishers.iter().count() }</strong> { " publishers" }
                                </p>
                            </div>
                        </div>
                        <div class="level-right">
                            <p class="level-item">
                                <a class="button is-success">{ "New" }</a>
                            </p>
                        </div>
                    </nav>
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "Name" }</th>
                                <th>{ "Short Name" }</th>
                                <th>{ "URL" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for body.data.publishers.iter().map(|p| self.render_publisher(p)) }
                        </tbody>
                    </table>
                </>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}

impl PublishersComponent {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }

    fn render_publisher(&self, p: &Publisher) -> Html {
        html! {
            <tr
                class="row"
                onclick=&self.change_route(AppRoute::Admin(AdminRoute::Publisher(p.publisher_id.clone())))
            >
                <td>{&p.publisher_id}</td>
                <td>{&p.publisher_name}</td>
                <td>{&p.publisher_shortname.clone().unwrap_or("".to_string())}</td>
                <td>{&p.publisher_url.clone().unwrap_or("".to_string())}</td>
            </tr>
        }
    }
}
