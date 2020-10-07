use std::str::FromStr;
use thoth_api::models::work::WorkStatus;
use thoth_api::models::work::WorkType;
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
use crate::api::models::Contribution;
use crate::api::models::Imprint;
use crate::api::models::Issue;
use crate::api::models::Publication;
use crate::api::models::Work;
use crate::api::models::WorkStatusValues;
use crate::api::models::WorkTypeValues;
use crate::api::work_query::FetchActionWork;
use crate::api::work_query::FetchWork;
use crate::api::work_query::Variables;
use crate::api::work_query::WorkRequest;
use crate::api::work_query::WorkRequestBody;
use crate::api::work_query::WORK_QUERY;
use crate::component::contributions_form::ContributionsFormComponent;
use crate::component::issues_form::IssuesFormComponent;
use crate::component::publications_form::PublicationsFormComponent;
use crate::component::utils::FormDateInput;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormWorkStatusSelect;
use crate::component::utils::FormWorkTypeSelect;
use crate::component::utils::Loader;
use crate::string::SAVE_BUTTON;

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
    UpdateContributions(Option<Vec<Contribution>>),
    UpdatePublications(Option<Vec<Publication>>),
    UpdateIssues(Option<Vec<Issue>>),
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
                contributor_id: None,
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
                            None => Default::default(),
                        };
                        self.data.imprints = body.data.imprints.to_owned();
                        self.data.work_types = body.data.work_types.enum_values.to_owned();
                        self.data.work_statuses = body.data.work_statuses.enum_values.to_owned();
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetWork => {
                self.link
                    .send_future(self.fetch_work.fetch(Msg::SetWorkFetchState));
                self.link
                    .send_message(Msg::SetWorkFetchState(FetchAction::Fetching));
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
            Msg::ChangeImprint(imprint_id) => {
                // we already have the full list of imprints
                if let Some(index) = self
                    .data
                    .imprints
                    .iter()
                    .position(|i| i.imprint_id == imprint_id)
                {
                    self.work
                        .imprint
                        .neq_assign(self.data.imprints.get(index).unwrap().clone())
                } else {
                    false
                }
            }
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
            Msg::UpdateContributions(contributions) => {
                self.work.contributions.neq_assign(contributions)
            }
            Msg::UpdatePublications(publications) => {
                self.work.publications.neq_assign(publications)
            }
            Msg::UpdateIssues(issues) => {
                self.work.issues.neq_assign(issues)
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
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = self.link.callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::Save
                });
                html! {
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
                                    value=&self.work.imprint.imprint_id
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
                        <ContributionsFormComponent
                            contributions=&self.work.contributions
                            work_id=&self.work.work_id
                            update_contributions=self.link.callback(|c: Option<Vec<Contribution>>| Msg::UpdateContributions(c))
                        />
                        <PublicationsFormComponent
                            publications=&self.work.publications
                            work_id=&self.work.work_id
                            update_publications=self.link.callback(|p: Option<Vec<Publication>>| Msg::UpdatePublications(p))
                        />
                        <IssuesFormComponent
                            issues=&self.work.issues
                            work_id=&self.work.work_id
                            update_issues=self.link.callback(|i: Option<Vec<Issue>>| Msg::UpdateIssues(i))
                        />

                        <div class="field">
                            <div class="control">
                                <button class="button is-success" type="submit">
                                    { SAVE_BUTTON }
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
