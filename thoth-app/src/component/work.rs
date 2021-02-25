use std::str::FromStr;
use thoth_api::account::model::AccountDetails;
use thoth_api::work::model::WorkStatus;
use thoth_api::work::model::WorkType;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::contributions_form::ContributionsFormComponent;
use crate::component::fundings_form::FundingsFormComponent;
use crate::component::issues_form::IssuesFormComponent;
use crate::component::languages_form::LanguagesFormComponent;
use crate::component::publications_form::PublicationsFormComponent;
use crate::component::subjects_form::SubjectsFormComponent;
use crate::component::utils::FormDateInput;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormWorkStatusSelect;
use crate::component::utils::FormWorkTypeSelect;
use crate::component::utils::Loader;
use crate::models::contribution::Contribution;
use crate::models::funding::Funding;
use crate::models::imprint::Imprint;
use crate::models::issue::Issue;
use crate::models::language::Language;
use crate::models::publication::Publication;
use crate::models::subject::Subject;
use crate::models::work::delete_work_mutation::DeleteWorkRequest;
use crate::models::work::delete_work_mutation::DeleteWorkRequestBody;
use crate::models::work::delete_work_mutation::PushActionDeleteWork;
use crate::models::work::delete_work_mutation::PushDeleteWork;
use crate::models::work::delete_work_mutation::Variables as DeleteVariables;
use crate::models::work::update_work_mutation::PushActionUpdateWork;
use crate::models::work::update_work_mutation::PushUpdateWork;
use crate::models::work::update_work_mutation::UpdateWorkRequest;
use crate::models::work::update_work_mutation::UpdateWorkRequestBody;
use crate::models::work::update_work_mutation::Variables as UpdateVariables;
use crate::models::work::work_query::FetchActionWork;
use crate::models::work::work_query::FetchWork;
use crate::models::work::work_query::Variables;
use crate::models::work::work_query::WorkRequest;
use crate::models::work::work_query::WorkRequestBody;
use crate::models::work::Work;
use crate::models::work::WorkStatusValues;
use crate::models::work::WorkTypeValues;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::DELETE_BUTTON;
use crate::string::SAVE_BUTTON;

pub struct WorkComponent {
    work: Work,
    data: WorkFormData,
    fetch_work: FetchWork,
    push_work: PushUpdateWork,
    delete_work: PushDeleteWork,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
    current_user: AccountDetails,
}

#[derive(Default)]
struct WorkFormData {
    imprints: Vec<Imprint>,
    work_types: Vec<WorkTypeValues>,
    work_statuses: Vec<WorkStatusValues>,
}

pub enum Msg {
    SetWorkFetchState(FetchActionWork),
    GetWork,
    SetWorkPushState(PushActionUpdateWork),
    UpdateWork,
    SetWorkDeleteState(PushActionDeleteWork),
    DeleteWork,
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
    UpdateContributions(Option<Vec<Contribution>>),
    UpdateFundings(Option<Vec<Funding>>),
    UpdatePublications(Option<Vec<Publication>>),
    UpdateLanguages(Option<Vec<Language>>),
    UpdateSubjects(Option<Vec<Subject>>),
    UpdateIssues(Option<Vec<Issue>>),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub work_id: String,
    pub current_user: AccountDetails,
}

impl Component for WorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let publishers = props.current_user.resource_access.restricted_to();
        let body = WorkRequestBody {
            variables: Variables {
                work_id: Some(props.work_id),
                publishers,
            },
            ..Default::default()
        };
        let request = WorkRequest { body };
        let fetch_work = Fetch::new(request);
        let push_work = Default::default();
        let delete_work = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let work: Work = Default::default();
        let data: WorkFormData = Default::default();
        let router = RouteAgentDispatcher::new();
        let current_user = props.current_user;

        link.send_message(Msg::GetWork);

        WorkComponent {
            work,
            data,
            fetch_work,
            push_work,
            delete_work,
            link,
            router,
            notification_bus,
            current_user,
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
            Msg::SetWorkPushState(fetch_state) => {
                self.push_work.apply(fetch_state);
                match self.push_work.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_work {
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
            Msg::UpdateWork => {
                let body = UpdateWorkRequestBody {
                    variables: UpdateVariables {
                        work_id: self.work.work_id.clone(),
                        work_type: self.work.work_type.clone(),
                        work_status: self.work.work_status.clone(),
                        full_title: self.work.full_title.clone(),
                        title: self.work.title.clone(),
                        subtitle: self.work.subtitle.clone(),
                        reference: self.work.reference.clone(),
                        edition: self.work.edition,
                        imprint_id: self.work.imprint.imprint_id.clone(),
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
                        lccn: self.work.lccn.clone(),
                        oclc: self.work.oclc.clone(),
                        short_abstract: self.work.short_abstract.clone(),
                        long_abstract: self.work.long_abstract.clone(),
                        general_note: self.work.general_note.clone(),
                        toc: self.work.toc.clone(),
                        cover_url: self.work.cover_url.clone(),
                        cover_caption: self.work.cover_caption.clone(),
                    },
                    ..Default::default()
                };
                let request = UpdateWorkRequest { body };
                self.push_work = Fetch::new(request);
                self.link
                    .send_future(self.push_work.fetch(Msg::SetWorkPushState));
                self.link
                    .send_message(Msg::SetWorkPushState(FetchAction::Fetching));
                false
            }
            Msg::SetWorkDeleteState(fetch_state) => {
                self.delete_work.apply(fetch_state);
                match self.delete_work.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_work {
                        Some(w) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Deleted {}", w.title),
                                NotificationStatus::Success,
                            )));
                            self.link
                                .send_message(Msg::ChangeRoute(AppRoute::Admin(AdminRoute::Works)));
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
            Msg::DeleteWork => {
                let body = DeleteWorkRequestBody {
                    variables: DeleteVariables {
                        work_id: self.work.work_id.clone(),
                    },
                    ..Default::default()
                };
                let request = DeleteWorkRequest { body };
                self.delete_work = Fetch::new(request);
                self.link
                    .send_future(self.delete_work.fetch(Msg::SetWorkDeleteState));
                self.link
                    .send_message(Msg::SetWorkDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeTitle(title) => {
                if self.work.title.neq_assign(title.trim().to_owned()) {
                    self.work.full_title = self.work.compile_fulltitle();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeSubtitle(value) => {
                let subtitle = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                if self.work.subtitle.neq_assign(subtitle) {
                    self.work.full_title = self.work.compile_fulltitle();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeWorkType(work_type) => self.work.work_type.neq_assign(work_type),
            Msg::ChangeWorkStatus(work_status) => self.work.work_status.neq_assign(work_status),
            Msg::ChangeReference(value) => {
                let reference = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.reference.neq_assign(reference)
            }
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
            Msg::ChangeDoi(value) => {
                let doi = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.doi.neq_assign(doi)
            }
            Msg::ChangeDate(date) => self.work.publication_date.neq_assign(Some(date)),
            Msg::ChangePlace(value) => {
                let place = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.place.neq_assign(place)
            }
            Msg::ChangeWidth(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let width = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.width.neq_assign(width)
            }
            Msg::ChangeHeight(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let height = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.height.neq_assign(height)
            }
            Msg::ChangePageCount(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let page_count = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.page_count.neq_assign(page_count)
            }
            Msg::ChangePageBreakdown(value) => {
                let breakdown = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.page_breakdown.neq_assign(breakdown)
            }
            Msg::ChangeImageCount(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let image_count = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.image_count.neq_assign(image_count)
            }
            Msg::ChangeTableCount(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let table_count = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.table_count.neq_assign(table_count)
            }
            Msg::ChangeAudioCount(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let audio_count = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.audio_count.neq_assign(audio_count)
            }
            Msg::ChangeVideoCount(value) => {
                let count: i32 = value.parse().unwrap_or(0);
                let video_count = match count == 0 {
                    true => None,
                    false => Some(count),
                };
                self.work.video_count.neq_assign(video_count)
            }
            Msg::ChangeLicense(value) => {
                let license = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.license.neq_assign(license)
            }
            Msg::ChangeCopyright(copyright) => self
                .work
                .copyright_holder
                .neq_assign(copyright.trim().to_owned()),
            Msg::ChangeLandingPage(value) => {
                let landing_page = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.landing_page.neq_assign(landing_page)
            }
            Msg::ChangeLccn(value) => {
                let lccn = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.lccn.neq_assign(lccn)
            }
            Msg::ChangeOclc(value) => {
                let oclc = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.oclc.neq_assign(oclc)
            }
            Msg::ChangeShortAbstract(value) => {
                let short_abstract = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.short_abstract.neq_assign(short_abstract)
            }
            Msg::ChangeLongAbstract(value) => {
                let long_abstract = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.long_abstract.neq_assign(long_abstract)
            }
            Msg::ChangeNote(value) => {
                let note = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.general_note.neq_assign(note)
            }
            Msg::ChangeToc(value) => {
                let toc = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.toc.neq_assign(toc)
            }
            Msg::ChangeCoverUrl(value) => {
                let cover_url = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.cover_url.neq_assign(cover_url)
            }
            Msg::ChangeCoverCaption(value) => {
                let cover_caption = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.work.cover_caption.neq_assign(cover_caption)
            }
            Msg::UpdateContributions(contributions) => {
                self.work.contributions.neq_assign(contributions)
            }
            Msg::UpdateFundings(fundings) => self.work.fundings.neq_assign(fundings),
            Msg::UpdatePublications(publications) => {
                self.work.publications.neq_assign(publications)
            }
            Msg::UpdateLanguages(languages) => self.work.languages.neq_assign(languages),
            Msg::UpdateSubjects(subjects) => self.work.subjects.neq_assign(subjects),
            Msg::UpdateIssues(issues) => self.work.issues.neq_assign(issues),
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
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
                    Msg::UpdateWork
                });
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit work" }
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                    <button class="button is-danger" onclick=self.link.callback(|_| Msg::DeleteWork)>
                                        { DELETE_BUTTON }
                                    </button>
                                </p>
                            </div>
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
                                    <FormTextInput
                                        label = "LCCN"
                                        value=&self.work.lccn
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeLccn(e.value))
                                    />
                                    <FormTextInput
                                        label = "OCLC Number"
                                        value=&self.work.oclc
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
                                        value=&self.work.width
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeWidth(e.value))
                                    />
                                    <FormNumberInput
                                        label = "Height"
                                        value=&self.work.height
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeHeight(e.value))
                                    />
                                    <FormNumberInput
                                        label = "Page Count"
                                        value=&self.work.page_count
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
                                        value=&self.work.image_count
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeImageCount(e.value))
                                    />
                                    <FormNumberInput
                                        label = "Table Count"
                                        value=&self.work.table_count
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeTableCount(e.value))
                                    />
                                    <FormNumberInput
                                        label = "Audio Count"
                                        value=&self.work.audio_count
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeAudioCount(e.value))
                                    />
                                    <FormNumberInput
                                        label = "Video Count"
                                        value=&self.work.video_count
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeVideoCount(e.value))
                                    />
                                </div>
                            </div>
                            <FormUrlInput
                                label = "License"
                                value=&self.work.license
                                oninput=self.link.callback(|e: InputData| Msg::ChangeLicense(e.value))
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

                        <hr/>

                        <article class="message is-info">
                            <div class="message-body">
                                { "Relations below are saved automatically upon change." }
                            </div>
                        </article>

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
                        <LanguagesFormComponent
                            languages=&self.work.languages
                            work_id=&self.work.work_id
                            update_languages=self.link.callback(|l: Option<Vec<Language>>| Msg::UpdateLanguages(l))
                        />
                        <SubjectsFormComponent
                            subjects=&self.work.subjects
                            work_id=&self.work.work_id
                            update_subjects=self.link.callback(|s: Option<Vec<Subject>>| Msg::UpdateSubjects(s))
                        />
                        <IssuesFormComponent
                            issues=&self.work.issues
                            work_id=&self.work.work_id
                            current_user=&self.current_user
                            update_issues=self.link.callback(|i: Option<Vec<Issue>>| Msg::UpdateIssues(i))
                        />
                        <FundingsFormComponent
                            fundings=&self.work.fundings
                            work_id=&self.work.work_id
                            update_fundings=self.link.callback(|f: Option<Vec<Funding>>| Msg::UpdateFundings(f))
                        />
                    </>
                }
            }
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
