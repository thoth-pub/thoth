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

use crate::component::utils::Loader;

pub struct WorkComponent {
    markdown: Fetch<Request, ResponseBody>,
    link: ComponentLink<Self>,
    props: Props,
}

pub enum Msg {
    SetMarkdownFetchState(FetchAction<ResponseBody>),
    GetMarkdown,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub work_id: String,
}

#[derive(Default, Debug, Clone)]
pub struct Request {
    body: RequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    work_id: String,
    title: String,
    subtitle: Option<String>,
    doi: String,
    contributions: Option<Vec<Contribution>>,
    imprint: Imprint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Imprint {
    publisher: Publisher,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    publisher_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    main_contribution: bool,
    contributor: Contributor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseBody {
    data: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    work: Work,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestBody {
    query: String,
    variables: String,
}

impl Default for ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            data: Default::default(),
        }
    }
}

impl Default for ResponseData {
    fn default() -> ResponseData {
        ResponseData {
            work: Work {
                work_id: "".to_string(),
                title: "".to_string(),
                subtitle: None,
                doi: "".to_string(),
                contributions: None,
                imprint: Imprint { publisher: Publisher {publisher_name: "".to_string()}},
            },
        }
    }
}

impl Default for RequestBody {
    fn default() -> RequestBody {
        RequestBody { query: "".to_string(), variables: "null".to_string() }
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

impl Component for WorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let query = format!("
            {{
            work(workId: \"{}\") {{
                title
                subtitle
                workId
                doi
                contributions {{
                    mainContribution
                    contributor {{
                        fullName
                    }}
                }}
                imprint {{
                    publisher {{
                        publisherName
                    }}
                }}
            }}
        }}
        ", &props.work_id);
        let body = RequestBody { query, variables: "null".to_string()};
        let request = Request { body };
        let markdown = Fetch::new(request);

        WorkComponent {
            markdown,
            link,
            props,
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
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => {
                let w = &body.data.work;
                let subtitle = w.subtitle.clone().unwrap_or("".to_string());
                html! {
                    <>
                        <div class="field">
                            <label class="label">{"Title"}</label>
                            <div class="control">
                                <input
                                    class="input"
                                    type="text"
                                    placeholder="Title"
                                    value={&w.title}
                                />
                            </div>
                        </div>

                        <div class="field">
                            <label class="label">{"Subtitle"}</label>
                            <div class="control">
                                <input
                                    class="input"
                                    type="text"
                                    placeholder="Subtitle"
                                    value={subtitle}
                                />
                            </div>
                        </div>

                        <div class="field">
                            <label class="label">{"Subject"}</label>
                            <div class="control">
                                <div class="select">
                                <select>
                                    <option>{"Select dropdown"}</option>
                                    <option>{"With options"}</option>
                                </select>
                                </div>
                            </div>
                        </div>

                        <div class="field">
                            <label class="label">{"Message"}</label>
                            <div class="control">
                                <textarea class="textarea" placeholder="Textarea"></textarea>
                            </div>
                        </div>

                        <div class="field">
                            <div class="control">
                                <button class="button is-success">{"Save"}</button>
                            </div>
                        </div>
                    </>
                }
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
