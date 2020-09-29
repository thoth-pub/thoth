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
use crate::api::models::Work;
use crate::api::models::Imprint;
use crate::api::models::WorkTypeValues;
use crate::api::models::WorkStatusValues;
use crate::api::work_query::FetchActionWork;
use crate::api::work_query::FetchWork;
use crate::api::work_query::Variables;
use crate::api::work_query::WorkRequest;
use crate::api::work_query::WorkRequestBody;
use crate::api::work_query::WORK_QUERY;
use crate::component::contributions_form::ContributionsFormComponent;
use crate::component::utils::FormDateInput;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormWorkStatusSelect;
use crate::component::utils::FormWorkTypeSelect;
use crate::component::utils::Loader;

pub struct WorkComponent {
    work: Work,
    data: WorkFormData,
    fetch_work: FetchWork,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

struct WorkFormData {
    imprints: Vec<Imprint>,
    work_types: Vec<WorkTypeValues>,
    work_statuses: Vec<WorkStatusValues>,
}

pub enum Msg {
    SetWorkFetchState(FetchActionWork),
    GetWork,
    Save,
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
            variables: Variables {
                work_id: Some(props.work_id),
                filter: None,
            },
        };
        let request = WorkRequest { body };
        let fetch_work = Fetch::new(request);
        let notification_bus = NotificationBus::dispatcher();
        let work: Work = Default::default();
        let data = WorkFormData {
            imprints: vec![],
            work_types: vec![],
            work_statuses: vec![],
        };

        link.send_message(Msg::GetWork);

        WorkComponent {
            work,
            data,
            fetch_work,
            link,
            notification_bus,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.fetch_work.fetch(Msg::SetWorkFetchState));
            self.link
                .send_message(Msg::SetWorkFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetWorkFetchState(fetch_state) => {
                self.fetch_work.apply(fetch_state);
                match self.fetch_work.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        self.work = match &body.data.work {
                            Some(w) => w.to_owned(),
                            None => Default::default()
                        };
                        self.data.imprints = body.data.imprints.to_owned();
                        self.data.work_types = body.data.work_types.enum_values.to_owned();
                        self.data.work_statuses = body.data.work_statuses.enum_values.to_owned();
                        true
                    },
                    FetchState::Failed(_, _err) => false
                }
            }
            Msg::GetWork => {
                self.link
                    .send_future(self.fetch_work.fetch(Msg::SetWorkFetchState));
                self.link
                    .send_message(Msg::SetWorkFetchState(FetchAction::Fetching));
                false
            }
            Msg::Save => {
                log::debug!("{:?}", self.work);
                self.notification_bus.send(Request::NotificationBusMsg((
                    "Saved".to_string(),
                    NotificationStatus::Success,
                )));
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match self.fetch_work.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {
                    <div class="buttons has-addons is-centered">
                        <button
                            class="button is-success is-large"
                            onclick=self.link.callback(|_| Msg::GetWork)
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
            FetchState::Fetched(_body) => {
                let callback = self.link.callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::Save
                });
                html! {
                    <form onsubmit=callback>
                        <FormTextInput label = "Title" value=&self.work.title required = true />
                        <FormTextInput label = "Subtitle" value=&self.work.subtitle />
                        <FormWorkTypeSelect
                            label = "Work Type"
                            value=&self.work.work_type
                            data=&self.data.work_types
                            required = true
                        />
                        <FormWorkStatusSelect
                            label = format!("Work Status: {}", self.work.work_status)
                            value=&self.work.work_status
                            data=&self.data.work_statuses
                            required = true
                        />
                        <FormTextInput label = "Internal Reference" value=&self.work.reference />
                        <FormImprintSelect
                            label = "Imprint"
                            value=&self.work.imprint.imprint_id
                            data=&self.data.imprints
                            required = true
                        />
                        <FormNumberInput label = "Edition" value=&self.work.edition required = true />
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
                        <FormNumberInput label = "OCLC Number" value=self.work.oclc />
                        <FormTextarea label = "Short Abstract" value=&self.work.short_abstract />
                        <FormTextarea label = "Long Abstract" value=&self.work.long_abstract />
                        <FormTextarea label = "General Note" value=&self.work.general_note />
                        <FormTextarea label = "Table of Content" value=&self.work.toc />
                        <FormUrlInput label = "Cover URL" value=&self.work.cover_url />
                        <FormTextarea label = "Cover Caption" value=&self.work.cover_caption />
                        <ContributionsFormComponent
                            contributions=&self.work.contributions
                        />

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
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
