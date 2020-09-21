use yew::html;
use yew::prelude::*;
use yew::ComponentLink;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::api::models::Work;

pub struct WorkFormComponent {
    work: Work,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    ClickedSave,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub work: Work,
}

impl Component for WorkFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let work = props.work;
        let notification_bus = NotificationBus::dispatcher();

        WorkFormComponent {
            work,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
        html! {
            <form>
                <div class="field">
                    <label class="label">{"Title"}</label>
                    <div class="control">
                        <input
                            class="input"
                            type="text"
                            placeholder="Title"
                            value={&self.work.title}
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
                            value={self.work.subtitle.clone().unwrap_or("".to_string())}
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
                            value={self.work.reference.clone().unwrap_or("".to_string())}
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
                            value={self.work.edition}
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
                            value={self.work.doi.clone().unwrap_or("".to_string())}
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
                            value={self.work.publication_date.clone().unwrap_or("".to_string())}
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
                            value={self.work.place.clone().unwrap_or("".to_string())}
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
                            value={self.work.width.clone().unwrap_or(0)}
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
                            value={self.work.height.clone().unwrap_or(0)}
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
                            value={self.work.page_count.clone().unwrap_or(0)}
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
                            value={self.work.page_breakdown.clone().unwrap_or("".to_string())}
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
                            value={self.work.image_count.clone().unwrap_or(0)}
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
                            value={self.work.table_count.clone().unwrap_or(0)}
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
                            value={self.work.audio_count.clone().unwrap_or(0)}
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
                            value={self.work.video_count.clone().unwrap_or(0)}
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
                            value={&self.work.copyright_holder}
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
                            value={self.work.landing_page.clone().unwrap_or("".to_string())}
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
                            value={self.work.lccn.clone().unwrap_or(0)}
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
                            value={self.work.oclc.clone().unwrap_or(0)}
                        />
                    </div>
                </div>

                <div class="field">
                    <label class="label">{"Short Abstract"}</label>
                    <div class="control">
                        <textarea class="textarea" placeholder="Short Abstract">
                        {self.work.short_abstract.clone().unwrap_or("".to_string())}
                        </textarea>
                    </div>
                </div>

                <div class="field">
                    <label class="label">{"Long Abstract"}</label>
                    <div class="control">
                        <textarea class="textarea" placeholder="Long Abstract">
                        {self.work.long_abstract.clone().unwrap_or("".to_string())}
                        </textarea>
                    </div>
                </div>

                <div class="field">
                    <label class="label">{"General Note"}</label>
                    <div class="control">
                        <textarea class="textarea" placeholder="General Note">
                        {self.work.general_note.clone().unwrap_or("".to_string())}
                        </textarea>
                    </div>
                </div>

                <div class="field">
                    <label class="label">{"Table of Content"}</label>
                    <div class="control">
                        <textarea class="textarea" placeholder="Table of Content">
                        {self.work.toc.clone().unwrap_or("".to_string())}
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
                            value={self.work.cover_url.clone().unwrap_or("".to_string())}
                        />
                    </div>
                </div>

                <div class="field">
                    <label class="label">{"Cover Caption"}</label>
                    <div class="control">
                        <textarea class="textarea" placeholder="Cover Caption">
                        {self.work.cover_caption.clone().unwrap_or("".to_string())}
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
    }
}
