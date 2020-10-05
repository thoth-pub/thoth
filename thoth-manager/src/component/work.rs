use std::str::FromStr;
use thoth_api::models::contributor::ContributionType;
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
use crate::api::models::Contributor;
use crate::api::models::Imprint;
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
    institution_value: String,
    biography_value: String,
    contributiontype_value: ContributionType,
    maincontribution_value: bool,
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
    AddContribution(Contributor),
    RemoveContribution(String),
    ChangeInstitutionEditValue(String),
    ChangeInstitution(String),
    ChangeBiographyEditValue(String),
    ChangeBiography(String),
    ChangeContributiontypeEditValue(ContributionType),
    ChangeContributiontype(String),
    ChangeMainContributionEditValue(bool),
    ChangeMainContribution(String),
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
        let institution_value = "".into();
        let biography_value = "".into();
        let contributiontype_value = ContributionType::Author;
        let maincontribution_value = false;
        let data = WorkFormData {
            imprints: vec![],
            work_types: vec![],
            work_statuses: vec![],
        };

        link.send_message(Msg::GetWork);

        WorkComponent {
            work,
            institution_value,
            biography_value,
            contributiontype_value,
            maincontribution_value,
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
            Msg::AddContribution(contributor) => {
                let mut contributions: Vec<Contribution> =
                    self.work.contributions.clone().unwrap_or_default();
                let contributor_id = contributor.contributor_id.clone();
                let contribution = Contribution {
                    work_id: self.work.work_id.clone(),
                    contributor_id: contributor_id.clone(),
                    contribution_type: ContributionType::Author,
                    main_contribution: false,
                    biography: None,
                    institution: None,
                    contributor: Contributor {
                        contributor_id,
                        first_name: contributor.first_name,
                        last_name: contributor.last_name,
                        full_name: contributor.full_name,
                        orcid: contributor.orcid,
                        website: contributor.website,
                    },
                };
                contributions.push(contribution);
                self.work.contributions = Some(contributions);
                true
            }
            Msg::RemoveContribution(contributor_id) => {
                let to_keep: Vec<Contribution> = self
                    .work
                    .contributions
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|c| c.contributor_id != contributor_id)
                    .collect();
                self.work.contributions = Some(to_keep);
                true
            }
            Msg::ChangeInstitutionEditValue(institution) => {
                self.institution_value.neq_assign(institution);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeInstitution(contributor_id) => {
                let institution_value = self.institution_value.trim().to_string();
                let institution = match institution_value.is_empty() {
                    true => None,
                    false => Some(institution_value),
                };
                let mut contributions: Vec<Contribution> =
                    self.work.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.institution = institution;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.work.contributions = Some(contributions);
                    self.institution_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeBiographyEditValue(biography) => {
                self.biography_value.neq_assign(biography);
                false
            }
            Msg::ChangeBiography(contributor_id) => {
                let biography_value = self.biography_value.trim().to_string();
                let biography = match biography_value.is_empty() {
                    true => None,
                    false => Some(biography_value),
                };
                let mut contributions: Vec<Contribution> =
                    self.work.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.biography = biography;
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.work.contributions = Some(contributions);
                    self.biography_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeContributiontype(contributor_id) => {
                let mut contributions: Vec<Contribution> =
                    self.work.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.contribution_type = self.contributiontype_value.clone();
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.work.contributions = Some(contributions);
                    self.contributiontype_value = ContributionType::Author;
                    true
                } else {
                    false
                }
            }
            Msg::ChangeContributiontypeEditValue(contribution_type) => {
                self.contributiontype_value.neq_assign(contribution_type)
            }
            Msg::ChangeMainContribution(contributor_id) => {
                let mut contributions: Vec<Contribution> =
                    self.work.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.main_contribution = self.maincontribution_value.clone();
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.work.contributions = Some(contributions);
                    self.maincontribution_value = false;
                    true
                } else {
                    false
                }
            }
            Msg::ChangeMainContributionEditValue(main_contribution) => {
                self.maincontribution_value.neq_assign(main_contribution)
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
                        <FormTextInput
                            label = "Internal Reference"
                            oninput=self.link.callback(|e: InputData| Msg::ChangeReference(e.value))
                            value=&self.work.reference
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
                        <FormNumberInput
                            label = "Edition"
                            value=&self.work.edition
                            oninput=self.link.callback(|e: InputData| Msg::ChangeEdition(e.value))
                            required = true
                        />
                        <FormUrlInput
                            label = "Doi"
                            value=&self.work.doi
                            oninput=self.link.callback(|e: InputData| Msg::ChangeDoi(e.value))
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
                        <FormNumberInput
                            label = "Library of Congress Number (LCCN)"
                            value=self.work.lccn
                            oninput=self.link.callback(|e: InputData| Msg::ChangeLccn(e.value))
                        />
                        <FormNumberInput
                            label = "OCLC Number"
                            value=self.work.oclc
                            oninput=self.link.callback(|e: InputData| Msg::ChangeOclc(e.value))
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
                        <ContributionsFormComponent
                            contributions=&self.work.contributions
                            add_contribution=self.link.callback(|c: Contributor| Msg::AddContribution(c))
                            remove_contribution=self.link.callback(|id: String| Msg::RemoveContribution(id))
                            change_institution_value=self.link.callback(|e: InputData| Msg::ChangeInstitutionEditValue(e.value))
                            change_institution=self.link.callback(|id: String| Msg::ChangeInstitution(id))
                            change_biography_value=self.link.callback(|e: InputData| Msg::ChangeBiographyEditValue(e.value))
                            change_biography=self.link.callback(|id: String| Msg::ChangeBiography(id))
                            change_contributiontype_value=self.link.callback(|event| match event {
                                ChangeData::Select(elem) => {
                                    let value = elem.value();
                                    Msg::ChangeContributiontypeEditValue(ContributionType::from_str(&value).unwrap())
                                }
                                _ => unreachable!(),
                            })
                            change_contributiontype=self.link.callback(|id: String| Msg::ChangeContributiontype(id))
                            change_maincontribution_value=self.link.callback(|event| match event {
                                ChangeData::Select(elem) => {
                                    let value = elem.value();
                                    log::info!("Main: {}", value);
                                    let boolean = value == "true".to_string();
                                    Msg::ChangeMainContributionEditValue(boolean)
                                }
                                _ => unreachable!(),
                            })
                            change_maincontribution=self.link.callback(|id: String| Msg::ChangeMainContribution(id))
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
