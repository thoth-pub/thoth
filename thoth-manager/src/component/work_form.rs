use yew::html;
use yew::prelude::*;
use yew::ComponentLink;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::api::models::Work;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormDateInput;
use crate::component::utils::FormNumberInput;

pub struct WorkFormComponent {
    work: Work,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    Save,
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
            Msg::Save => {
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
        let callback = self.link.callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::Save
        });
        html! {
            <form onsubmit=callback>
                <FormTextInput label = "Title" value=&self.work.title required = true />
                <FormTextInput label = "Subtitle" value=&self.work.subtitle />
                <FormTextInput label = "Internal Reference" value=&self.work.reference />
                <FormNumberInput label = "Edition" value=self.work.edition required = true />
                <FormUrlInput label = "Doi" value=&self.work.doi />
                <FormDateInput label = "Publication Date" value=&self.work.publication_date />
                <FormTextInput label = "Place of Publication" value=&self.work.place />
                <FormNumberInput label = "Width" value=self.work.width />
                <FormNumberInput label = "Height" value=self.work.height />
                <FormNumberInput label = "Page Count" value=self.work.page_count />
                <FormTextInput label = "Page Breakdown" value=&self.work.page_breakdown />
                <FormNumberInput label = "Image Count" value=self.work.image_count />
                <FormNumberInput label = "Table Count" value=self.work.table_count />
                <FormNumberInput label = "Audio Count" value=self.work.audio_count />
                <FormNumberInput label = "Video Count" value=self.work.video_count />
                <FormTextInput label = "Copyright Holder" value=&self.work.copyright_holder required = true />
                <FormUrlInput label = "Landing Page" value=&self.work.landing_page />
                <FormNumberInput label = "Library of Congress Number (LCCN)" value=self.work.lccn />
                <FormNumberInput label = "OCLC Number" value=&self.work.oclc />
                <FormTextarea label = "Short Abstract" value=&self.work.short_abstract />
                <FormTextarea label = "Long Abstract" value=&self.work.long_abstract />
                <FormTextarea label = "General Note" value=&self.work.general_note />
                <FormTextarea label = "Table of Content" value=&self.work.toc />
                <FormUrlInput label = "Cover URL" value=&self.work.cover_url />
                <FormTextarea label = "Cover Caption" value=&self.work.cover_caption />

                <div class="field">
                    <div class="control">
                        <button class="button is-success" type="submit">
                            {"Save"}
                        </button>
                    </div>
                </div>
            </form>
        }
    }
}
