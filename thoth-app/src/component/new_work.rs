use std::str::FromStr;
use thoth_api::work::model::WorkStatus;
use thoth_api::work::model::WorkType;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormDateInput;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormWorkStatusSelect;
use crate::component::utils::FormWorkTypeSelect;
use crate::models::imprint::imprints_query::FetchActionImprints;
use crate::models::imprint::imprints_query::FetchImprints;
use crate::models::imprint::Imprint;
use crate::models::work::create_work_mutation::CreateWorkRequest;
use crate::models::work::create_work_mutation::CreateWorkRequestBody;
use crate::models::work::create_work_mutation::PushActionCreateWork;
use crate::models::work::create_work_mutation::PushCreateWork;
use crate::models::work::create_work_mutation::Variables;
use crate::models::work::work_statuses_query::FetchActionWorkStatuses;
use crate::models::work::work_statuses_query::FetchWorkStatuses;
use crate::models::work::work_types_query::FetchActionWorkTypes;
use crate::models::work::work_types_query::FetchWorkTypes;
use crate::models::work::Work;
use crate::models::work::WorkStatusValues;
use crate::models::work::WorkTypeValues;
use crate::string::SAVE_BUTTON;

pub struct NewWorkComponent {
    work: Work,
    imprint_id: String,
    push_work: PushCreateWork,
    data: WorkFormData,
    fetch_imprints: FetchImprints,
    fetch_work_types: FetchWorkTypes,
    fetch_work_statuses: FetchWorkStatuses,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct WorkFormData {
    imprints: Vec<Imprint>,
    work_types: Vec<WorkTypeValues>,
    work_statuses: Vec<WorkStatusValues>,
}

pub enum Msg {
    SetImprintsFetchState(FetchActionImprints),
    GetImprints,
    SetWorkTypesFetchState(FetchActionWorkTypes),
    GetWorkTypes,
    SetWorkStatusesFetchState(FetchActionWorkStatuses),
    GetWorkStatuses,
    SetWorkPushState(PushActionCreateWork),
    CreateWork,
    ChangeTitle(String),
    ChangeSubtitle(String),
    ChangeWorkType(WorkType),
    ChangeWorkStatus(WorkStatus),
    ChangeReference(String),
    ChangeImprint(String),
    ChangeEdition(String),
    ChangeDoi(String),
    ChangeDate(String),
    ChangePlace(String),
    ChangeWidth(String),
    ChangeHeight(String),
    ChangePageCount(String),
    ChangePageBreakdown(String),
    ChangeImageCount(String),
    ChangeTableCount(String),
    ChangeAudioCount(String),
    ChangeVideoCount(String),
    ChangeLicense(String),
    ChangeCopyright(String),
    ChangeLandingPage(String),
    ChangeLccn(String),
    ChangeOclc(String),
    ChangeShortAbstract(String),
    ChangeLongAbstract(String),
    ChangeNote(String),
    ChangeToc(String),
    ChangeCoverUrl(String),
    ChangeCoverCaption(String),
}

impl Component for NewWorkComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_work = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let work: Work = Default::default();
        let imprint_id: String = Default::default();
        let data: WorkFormData = Default::default();
        let fetch_imprints: FetchImprints = Default::default();
        let fetch_work_types: FetchWorkTypes = Default::default();
        let fetch_work_statuses: FetchWorkStatuses = Default::default();

        link.send_message(Msg::GetImprints);
        link.send_message(Msg::GetWorkTypes);
        link.send_message(Msg::GetWorkStatuses);

        NewWorkComponent {
            work,
            imprint_id,
            push_work,
            data,
            fetch_imprints,
            fetch_work_types,
            fetch_work_statuses,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetImprintsFetchState(fetch_state) => {
                self.fetch_imprints.apply(fetch_state);
                self.data.imprints = match self.fetch_imprints.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.imprints.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetImprints => {
                self.link
                    .send_future(self.fetch_imprints.fetch(Msg::SetImprintsFetchState));
                self.link
                    .send_message(Msg::SetImprintsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetWorkTypesFetchState(fetch_state) => {
                self.fetch_work_types.apply(fetch_state);
                self.data.work_types = match self.fetch_work_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.work_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetWorkTypes => {
                self.link
                    .send_future(self.fetch_work_types.fetch(Msg::SetWorkTypesFetchState));
                self.link
                    .send_message(Msg::SetWorkTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetWorkStatusesFetchState(fetch_state) => {
                self.fetch_work_statuses.apply(fetch_state);
                self.data.work_statuses = match self.fetch_work_statuses.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.work_statuses.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetWorkStatuses => {
                self.link.send_future(
                    self.fetch_work_statuses
                        .fetch(Msg::SetWorkStatusesFetchState),
                );
                self.link
                    .send_message(Msg::SetWorkStatusesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetWorkPushState(fetch_state) => {
                self.push_work.apply(fetch_state);
                match self.push_work.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_work {
                        Some(w) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", w.title),
                                NotificationStatus::Success,
                            )));
                            true
                        }
                        None => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateWork => {
                let body = CreateWorkRequestBody {
                    variables: Variables {
                        work_type: self.work.work_type.clone(),
                        work_status: self.work.work_status.clone(),
                        full_title: self.work.full_title.clone(),
                        title: self.work.title.clone(),
                        subtitle: self.work.subtitle.clone(),
                        reference: self.work.reference.clone(),
                        edition: self.work.edition,
                        doi: self.work.doi.clone(),
                        publication_date: self.work.publication_date.clone(),
                        place: self.work.place.clone(),
                        width: self.work.width,
                        height: self.work.height,
                        page_count: self.work.page_count,
                        page_breakdown: self.work.page_breakdown.clone(),
                        image_count: self.work.image_count,
                        table_count: self.work.table_count,
                        audio_count: self.work.audio_count,
                        video_count: self.work.video_count,
                        license: self.work.license.clone(),
                        copyright_holder: self.work.copyright_holder.clone(),
                        landing_page: self.work.landing_page.clone(),
                        lccn: self.work.lccn,
                        oclc: self.work.oclc,
                        short_abstract: self.work.short_abstract.clone(),
                        long_abstract: self.work.long_abstract.clone(),
                        general_note: self.work.general_note.clone(),
                        toc: self.work.toc.clone(),
                        cover_url: self.work.cover_url.clone(),
                        cover_caption: self.work.cover_caption.clone(),
                        imprint_id: self.imprint_id.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateWorkRequest { body };
                self.push_work = Fetch::new(request);
                self.link
                    .send_future(self.push_work.fetch(Msg::SetWorkPushState));
                self.link
                    .send_message(Msg::SetWorkPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeTitle(title) => {
                if self.work.title.neq_assign(title) {
                    self.work.full_title = self.work.compile_fulltitle();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeSubtitle(subtitle) => {
                if self.work.subtitle.neq_assign(Some(subtitle)) {
                    self.work.full_title = self.work.compile_fulltitle();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeWorkType(work_type) => self.work.work_type.neq_assign(work_type),
            Msg::ChangeWorkStatus(work_status) => self.work.work_status.neq_assign(work_status),
            Msg::ChangeReference(reference) => self.work.reference.neq_assign(Some(reference)),
            Msg::ChangeImprint(imprint_id) => self.imprint_id.neq_assign(imprint_id),
            Msg::ChangeEdition(edition) => {
                let edition: i32 = edition.parse().unwrap_or(1);
                self.work.edition.neq_assign(edition)
            }
            Msg::ChangeDoi(doi) => self.work.doi.neq_assign(Some(doi)),
            Msg::ChangeDate(date) => self.work.publication_date.neq_assign(Some(date)),
            Msg::ChangePlace(place) => self.work.place.neq_assign(Some(place)),
            Msg::ChangeWidth(width) => {
                let width: i32 = width.parse().unwrap_or(0);
                self.work.width.neq_assign(Some(width))
            }
            Msg::ChangeHeight(height) => {
                let height: i32 = height.parse().unwrap_or(0);
                self.work.height.neq_assign(Some(height))
            }
            Msg::ChangePageCount(page_count) => {
                let page_count: i32 = page_count.parse().unwrap_or(0);
                self.work.page_count.neq_assign(Some(page_count))
            }
            Msg::ChangePageBreakdown(breakdown) => {
                self.work.page_breakdown.neq_assign(Some(breakdown))
            }
            Msg::ChangeImageCount(image_count) => {
                let image_count: i32 = image_count.parse().unwrap_or(0);
                self.work.image_count.neq_assign(Some(image_count))
            }
            Msg::ChangeTableCount(table_count) => {
                let table_count: i32 = table_count.parse().unwrap_or(0);
                self.work.table_count.neq_assign(Some(table_count))
            }
            Msg::ChangeAudioCount(audio_count) => {
                let audio_count: i32 = audio_count.parse().unwrap_or(0);
                self.work.audio_count.neq_assign(Some(audio_count))
            }
            Msg::ChangeVideoCount(video_count) => {
                let video_count: i32 = video_count.parse().unwrap_or(0);
                self.work.video_count.neq_assign(Some(video_count))
            }
            Msg::ChangeLicense(license) => self.work.license.neq_assign(Some(license)),
            Msg::ChangeCopyright(copyright) => self.work.copyright_holder.neq_assign(copyright),
            Msg::ChangeLandingPage(landing_page) => {
                self.work.landing_page.neq_assign(Some(landing_page))
            }
            Msg::ChangeLccn(lccn) => {
                let lccn: i32 = lccn.parse().unwrap_or(0);
                self.work.lccn.neq_assign(Some(lccn))
            }
            Msg::ChangeOclc(oclc) => {
                let oclc: i32 = oclc.parse().unwrap_or(0);
                self.work.oclc.neq_assign(Some(oclc))
            }
            Msg::ChangeShortAbstract(short_abstract) => {
                self.work.short_abstract.neq_assign(Some(short_abstract))
            }
            Msg::ChangeLongAbstract(long_abstract) => {
                self.work.long_abstract.neq_assign(Some(long_abstract))
            }
            Msg::ChangeNote(note) => self.work.general_note.neq_assign(Some(note)),
            Msg::ChangeToc(toc) => self.work.toc.neq_assign(Some(toc)),
            Msg::ChangeCoverUrl(cover_url) => self.work.cover_url.neq_assign(Some(cover_url)),
            Msg::ChangeCoverCaption(cover_caption) => {
                self.work.cover_caption.neq_assign(Some(cover_caption))
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let callback = self.link.callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateWork
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New work" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit=callback>
                    <div class="field is-horizontal">
                        <div class="field-body">
                            <FormWorkTypeSelect
                                label = "Work Type"
                                value=&self.work.work_type
                                data=&self.data.work_types
                                onchange=self.link.callback(|event| match event {
                                    ChangeData::Select(elem) => {
                                        let value = elem.value();
                                        Msg::ChangeWorkType(WorkType::from_str(&value).unwrap())
                                    }
                                    _ => unreachable!(),
                                })
                                required = true
                            />
                            <FormWorkStatusSelect
                                label = "Work Status"
                                value=&self.work.work_status
                                data=&self.data.work_statuses
                                onchange=self.link.callback(|event| match event {
                                    ChangeData::Select(elem) => {
                                        let value = elem.value();
                                        Msg::ChangeWorkStatus(WorkStatus::from_str(&value).unwrap())
                                    }
                                    _ => unreachable!(),
                                })
                                required = true
                            />
                            <FormImprintSelect
                                label = "Imprint"
                                value=&self.imprint_id
                                data=&self.data.imprints
                                onchange=self.link.callback(|event| match event {
                                    ChangeData::Select(elem) => {
                                        let value = elem.value();
                                        Msg::ChangeImprint(value.clone())
                                    }
                                    _ => unreachable!(),
                                })
                                required = true
                            />
                        </div>
                    </div>
                    <FormTextInput
                        label = "Title"
                        value=&self.work.title
                        oninput=self.link.callback(|e: InputData| Msg::ChangeTitle(e.value))
                        required = true
                    />
                    <FormTextInput
                        label = "Subtitle"
                        value=&self.work.subtitle
                        oninput=self.link.callback(|e: InputData| Msg::ChangeSubtitle(e.value))
                    />
                    <FormNumberInput
                        label = "Edition"
                        value=&self.work.edition
                        oninput=self.link.callback(|e: InputData| Msg::ChangeEdition(e.value))
                        required = true
                    />
                    <FormDateInput
                        label = "Publication Date"
                        value=&self.work.publication_date
                        oninput=self.link.callback(|e: InputData| Msg::ChangeDate(e.value))
                    />
                    <FormTextInput
                        label = "Place of Publication"
                        value=&self.work.place
                        oninput=self.link.callback(|e: InputData| Msg::ChangePlace(e.value))
                    />
                    <div class="field">
                        <div class="tile is-ancestor">
                            <div class="tile is-2 is-parent">
                                <div class="tile is-child">
                                    <figure class="image is-fullwidth">
                                        <img
                                            src={&self.work.cover_url.clone().unwrap_or("".to_string())}
                                            loading="lazy"
                                        />
                                    </figure>
                                </div>
                            </div>
                            <div class="tile is-parent">
                                <div class="tile is-child">
                                    <FormUrlInput
                                        label = "Cover URL"
                                        value=&self.work.cover_url
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeCoverUrl(e.value))
                                    />
                                    <FormTextarea
                                        label = "Cover Caption"
                                        value=&self.work.cover_caption
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeCoverCaption(e.value))
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="field is-horizontal">
                        <div class="field-body">
                            <FormUrlInput
                                label = "DOI"
                                value=&self.work.doi
                                oninput=self.link.callback(|e: InputData| Msg::ChangeDoi(e.value))
                            />
                            <FormNumberInput
                                label = "LCCN"
                                value=self.work.lccn
                                oninput=self.link.callback(|e: InputData| Msg::ChangeLccn(e.value))
                            />
                            <FormNumberInput
                                label = "OCLC Number"
                                value=self.work.oclc
                                oninput=self.link.callback(|e: InputData| Msg::ChangeOclc(e.value))
                            />
                            <FormTextInput
                                label = "Internal Reference"
                                oninput=self.link.callback(|e: InputData| Msg::ChangeReference(e.value))
                                value=&self.work.reference
                            />
                        </div>
                    </div>
                    <div class="field is-horizontal">
                        <div class="field-body">
                            <FormNumberInput
                                label = "Width"
                                value=self.work.width
                                oninput=self.link.callback(|e: InputData| Msg::ChangeWidth(e.value))
                            />
                            <FormNumberInput
                                label = "Height"
                                value=self.work.height
                                oninput=self.link.callback(|e: InputData| Msg::ChangeHeight(e.value))
                            />
                            <FormNumberInput
                                label = "Page Count"
                                value=self.work.page_count
                                oninput=self.link.callback(|e: InputData| Msg::ChangePageCount(e.value))
                            />
                            <FormTextInput
                                label = "Page Breakdown"
                                value=&self.work.page_breakdown
                                oninput=self.link.callback(|e: InputData| Msg::ChangePageBreakdown(e.value))
                            />
                        </div>
                    </div>
                    <div class="field is-horizontal">
                        <div class="field-body">
                            <FormNumberInput
                                label = "Image Count"
                                value=self.work.image_count
                                oninput=self.link.callback(|e: InputData| Msg::ChangeImageCount(e.value))
                            />
                            <FormNumberInput
                                label = "Table Count"
                                value=self.work.table_count
                                oninput=self.link.callback(|e: InputData| Msg::ChangeTableCount(e.value))
                            />
                            <FormNumberInput
                                label = "Audio Count"
                                value=self.work.audio_count
                                oninput=self.link.callback(|e: InputData| Msg::ChangeAudioCount(e.value))
                            />
                            <FormNumberInput
                                label = "Video Count"
                                value=self.work.video_count
                                oninput=self.link.callback(|e: InputData| Msg::ChangeVideoCount(e.value))
                            />
                        </div>
                    </div>
                    <FormTextInput
                        label = "Copyright Holder"
                        value=&self.work.copyright_holder
                        oninput=self.link.callback(|e: InputData| Msg::ChangeCopyright(e.value))
                        required = true
                    />
                    <FormTextInput
                        label = "License"
                        value=&self.work.license
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLicense(e.value))
                    />
                    <FormUrlInput
                        label = "Landing Page"
                        value=&self.work.landing_page
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLandingPage(e.value))
                    />
                    <FormTextarea
                        label = "Short Abstract"
                        value=&self.work.short_abstract
                        oninput=self.link.callback(|e: InputData| Msg::ChangeShortAbstract(e.value))
                    />
                    <FormTextarea
                        label = "Long Abstract"
                        value=&self.work.long_abstract
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLongAbstract(e.value))
                    />
                    <FormTextarea
                        label = "General Note"
                        value=&self.work.general_note
                        oninput=self.link.callback(|e: InputData| Msg::ChangeNote(e.value))
                    />
                    <FormTextarea
                        label = "Table of Content"
                        value=&self.work.toc
                        oninput=self.link.callback(|e: InputData| Msg::ChangeToc(e.value))
                    />

                    <div class="field">
                        <div class="control">
                            <button class="button is-success" type="submit">
                                { SAVE_BUTTON }
                            </button>
                        </div>
                    </div>
                </form>
            </>
        }
    }
}
