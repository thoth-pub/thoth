use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::api::work_query::FetchWork;
use crate::api::work_query::FetchActionWork;
use crate::api::work_query::WorkRequest;
use crate::api::work_query::WorkRequestBody;
use crate::component::utils::Loader;

pub struct WorkComponent {
    markdown: FetchWork,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionWork),
    GetMarkdown,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub work_id: String,
}

impl Component for WorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let query = format!("
            {{
            work(workId: \"{}\") {{
                workId
                fullTitle
                title
                doi
                coverUrl
                license
                publicationDate
                place
                contributions {{
                    mainContribution
                    contributor {{
                        fullName
                    }}
                }}
                imprint {{
                    publisher {{
                        publisherId
                        publisherName
                        publisherShortname
                        publisherUrl
                    }}
                }}
            }}
        }}
        ", &props.work_id);
        let body = WorkRequestBody { query, variables: "null".to_string()};
        let request = WorkRequest { body };
        let markdown = Fetch::new(request);

        WorkComponent {
            markdown,
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
