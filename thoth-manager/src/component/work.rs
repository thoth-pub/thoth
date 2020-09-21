use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::api::work_query::WORK_QUERY;
use crate::api::work_query::Variables;
use crate::api::work_query::FetchWork;
use crate::api::work_query::FetchActionWork;
use crate::api::work_query::WorkRequest;
use crate::api::work_query::WorkRequestBody;
use crate::component::utils::Loader;

pub struct WorkComponent {
    markdown: FetchWork,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionWork),
    GetMarkdown,
    ClickedSave,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub work_id: String,
}

impl Component for WorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let body = WorkRequestBody {
            query: WORK_QUERY.to_string(),
            variables: Variables { work_id: Some(props.work_id) },
        };
        let request = WorkRequest { body };
        let markdown = Fetch::new(request);
        let notification_bus = NotificationBus::dispatcher();

        WorkComponent {
            markdown,
            link,
            notification_bus,
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
            Msg::ClickedSave => {
                self.notification_bus
                    .send(Request::NotificationBusMsg(
                            ("Saved".to_string(), NotificationStatus::Success)));
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
                if let Some(w) = &body.data.work {
                    html! {
                        <form>
                            <div class="field">
                                <label class="label">{"Title"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="Title"
                                        value={&w.title}
                                        required=true
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
                                        value={w.subtitle.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Internal Reference"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="Internal Reference"
                                        value={w.reference.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Edition"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Edition"
                                        value={&w.edition}
                                        required=true
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Doi"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="url"
                                        placeholder="Doi"
                                        value={w.doi.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Publication Date"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="date"
                                        placeholder="Publication Date"
                                        value={w.publication_date.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Place of Publication"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="Place of Publication"
                                        value={w.place.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Width"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Width"
                                        value={w.width.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Height"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Height"
                                        value={w.height.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Page Count"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Page Count"
                                        value={w.page_count.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Page Breakdown"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="Page Breakdown"
                                        value={w.page_breakdown.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Image Count"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Image Count"
                                        value={w.image_count.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Table Count"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Table Count"
                                        value={w.table_count.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Audio Count"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Audio Count"
                                        value={w.audio_count.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Video Count"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Video Count"
                                        value={w.video_count.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Copyright Holder"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="text"
                                        placeholder="Copyright Holder"
                                        value={&w.copyright_holder}
                                        required=true
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Landing Page"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="url"
                                        placeholder="Landing Page"
                                        value={w.landing_page.clone().unwrap_or("".to_string())}
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
                                <label class="label">{"Library of Congress Number (LCCN)"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="Library of Congress Number (LCCN)"
                                        value={w.lccn.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"OCLC Number"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="number"
                                        placeholder="OCLC Number"
                                        value={w.oclc.clone().unwrap_or(0)}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Short Abstract"}</label>
                                <div class="control">
                                    <textarea class="textarea" placeholder="Short Abstract">
                                    {w.short_abstract.clone().unwrap_or("".to_string())}
                                    </textarea>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Long Abstract"}</label>
                                <div class="control">
                                    <textarea class="textarea" placeholder="Long Abstract">
                                    {w.long_abstract.clone().unwrap_or("".to_string())}
                                    </textarea>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"General Note"}</label>
                                <div class="control">
                                    <textarea class="textarea" placeholder="General Note">
                                    {w.general_note.clone().unwrap_or("".to_string())}
                                    </textarea>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Table of Content"}</label>
                                <div class="control">
                                    <textarea class="textarea" placeholder="Table of Content">
                                    {w.toc.clone().unwrap_or("".to_string())}
                                    </textarea>
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Cover Url"}</label>
                                <div class="control">
                                    <input
                                        class="input"
                                        type="url"
                                        placeholder="Cover URL"
                                        value={w.cover_url.clone().unwrap_or("".to_string())}
                                    />
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{"Cover Caption"}</label>
                                <div class="control">
                                    <textarea class="textarea" placeholder="Cover Caption">
                                    {w.cover_caption.clone().unwrap_or("".to_string())}
                                    </textarea>
                                </div>
                            </div>

                            <div class="field">
                                <div class="control">
                                    <button
                                        class="button is-success"
                                        type="submit"
                                        onclick=self.link.callback(|_| Msg::ClickedSave)
                                    >
                                        {"Save"}
                                    </button>
                                </div>
                            </div>
                        </form>
                    }
                } else {
                    html!{{ "Work could not be found" }}
                }
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
