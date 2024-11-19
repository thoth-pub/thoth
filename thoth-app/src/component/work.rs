use std::str::FromStr;
use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::contribution::Contribution;
use thoth_api::model::funding::FundingWithInstitution;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::issue::IssueWithSeries;
use thoth_api::model::language::Language;
use thoth_api::model::publication::Publication;
use thoth_api::model::reference::Reference;
use thoth_api::model::subject::Subject;
use thoth_api::model::work::WorkStatus;
use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
use thoth_api::model::{Doi, DOI_DOMAIN};
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yew_router::history::History;
use yew_router::prelude::RouterScopeExt;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::contributions_form::ContributionsFormComponent;
use crate::component::delete_dialogue::ConfirmDeleteComponent;
use crate::component::fundings_form::FundingsFormComponent;
use crate::component::issues_form::IssuesFormComponent;
use crate::component::languages_form::LanguagesFormComponent;
use crate::component::new_chapter::NewChapterComponent;
use crate::component::publications_form::PublicationsFormComponent;
use crate::component::references_form::ReferencesFormComponent;
use crate::component::related_works_form::RelatedWorksFormComponent;
use crate::component::subjects_form::SubjectsFormComponent;
use crate::component::utils::FormDateInput;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextInputExtended;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormWorkStatusSelect;
use crate::component::utils::FormWorkTypeSelect;
use crate::component::utils::Loader;
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
use crate::models::work::WorkStatusValues;
use crate::models::work::WorkTypeValues;
use crate::route::AdminRoute;
use crate::string::RELATIONS_INFO;
use crate::string::SAVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct WorkComponent {
    work: WorkWithRelations,
    // Track the user-entered DOI string, which may not be validly formatted
    doi: String,
    doi_warning: String,
    // Track imprint stored in database, as distinct from imprint selected in dropdown
    imprint_id: Uuid,
    // Track work_type stored in database, as distinct from work_type selected in dropdown
    work_type: WorkType,
    data: WorkFormData,
    fetch_work: FetchWork,
    push_work: PushUpdateWork,
    delete_work: PushDeleteWork,
    notification_bus: NotificationDispatcher,
    // Store props values locally in order to test whether they have been updated on props change
    resource_access: AccountAccess,
    work_id: Uuid,
}

#[derive(Default)]
struct WorkFormData {
    imprints: Vec<ImprintWithPublisher>,
    work_types: Vec<WorkTypeValues>,
    work_statuses: Vec<WorkStatusValues>,
}

#[allow(clippy::large_enum_variant)]
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
    ChangeImprint(Uuid),
    ChangeEdition(String),
    ChangeDoi(String),
    ChangeDate(String),
    ChangeWithdrawnDate(String),
    ChangePlace(String),
    ChangePageCount(String),
    ChangePageBreakdown(String),
    ChangeFirstPage(String),
    ChangeLastPage(String),
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
    ChangeBibliographyNote(String),
    ChangeToc(String),
    ChangeCoverUrl(String),
    ChangeCoverCaption(String),
    UpdateRelatedWorks(Option<Vec<WorkRelationWithRelatedWork>>),
    UpdateContributions(Option<Vec<Contribution>>),
    UpdateFundings(Option<Vec<FundingWithInstitution>>),
    UpdatePublications(Option<Vec<Publication>>),
    UpdateLanguages(Option<Vec<Language>>),
    UpdateSubjects(Option<Vec<Subject>>),
    UpdateIssues(Option<Vec<IssueWithSeries>>),
    UpdateReferences(Option<Vec<Reference>>),
}

#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub work_id: Uuid,
    pub current_user: AccountDetails,
}

impl Component for WorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let fetch_work: FetchWork = Default::default();
        let push_work = Default::default();
        let delete_work = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let work: WorkWithRelations = Default::default();
        let doi = Default::default();
        let doi_warning = Default::default();
        let imprint_id = work.imprint.imprint_id;
        let work_type = work.work_type;
        let data: WorkFormData = Default::default();
        let resource_access = ctx.props().current_user.resource_access.clone();
        let work_id = ctx.props().work_id;

        ctx.link().send_message(Msg::GetWork);

        WorkComponent {
            work,
            doi,
            doi_warning,
            imprint_id,
            work_type,
            data,
            fetch_work,
            push_work,
            delete_work,
            notification_bus,
            resource_access,
            work_id,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                        // Initialise user-entered DOI variable to match DOI in database
                        self.doi = self.work.doi.clone().unwrap_or_default().to_string();
                        self.imprint_id = self.work.imprint.imprint_id;
                        self.work_type = self.work.work_type;
                        body.data.imprints.clone_into(&mut self.data.imprints);
                        body.data
                            .work_types
                            .enum_values
                            .clone_into(&mut self.data.work_types);
                        body.data
                            .work_statuses
                            .enum_values
                            .clone_into(&mut self.data.work_statuses);

                        // If user doesn't have permission to edit this object, redirect to dashboard
                        if let Some(publishers) =
                            ctx.props().current_user.resource_access.restricted_to()
                        {
                            if !publishers
                                .contains(&self.work.imprint.publisher.publisher_id.to_string())
                            {
                                ctx.link().history().unwrap().push(AdminRoute::Dashboard);
                            }
                        }
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetWork => {
                let body = WorkRequestBody {
                    variables: Variables {
                        work_id: Some(ctx.props().work_id),
                        publishers: ctx.props().current_user.resource_access.restricted_to(),
                    },
                    ..Default::default()
                };
                let request = WorkRequest { body };
                self.fetch_work = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_work.fetch(Msg::SetWorkFetchState));
                ctx.link()
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
                            // Save was successful: update user-entered DOI variable to match DOI in database
                            self.doi = self.work.doi.clone().unwrap_or_default().to_string();
                            self.doi_warning.clear();
                            self.imprint_id = self.work.imprint.imprint_id;
                            self.work_type = self.work.work_type;
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
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateWork => {
                // Only update the DOI value with the current user-entered string
                // if it is validly formatted - otherwise keep the database version.
                // If no DOI was provided, no format check is required.
                if self.doi.is_empty() {
                    self.work.doi.neq_assign(None);
                } else if let Ok(result) = self.doi.parse::<Doi>() {
                    self.work.doi.neq_assign(Some(result));
                }
                // Clear any fields which are not applicable to the currently selected work type.
                // (Do not clear them before the save point as the user may change the type again.)
                if self.work.work_type == WorkType::BookChapter {
                    self.work.edition = None;
                    self.work.toc = None;
                    self.work.lccn = None;
                    self.work.oclc = None;
                } else {
                    self.work.first_page = None;
                    self.work.last_page = None;
                    self.work.page_interval = None;
                }
                if self.work.work_status != WorkStatus::Withdrawn
                    && self.work.work_status != WorkStatus::Superseded
                {
                    self.work.withdrawn_date = None;
                }
                let body = UpdateWorkRequestBody {
                    variables: UpdateVariables {
                        work_id: self.work.work_id,
                        work_type: self.work.work_type,
                        work_status: self.work.work_status,
                        full_title: self.work.full_title.clone(),
                        title: self.work.title.clone(),
                        subtitle: self.work.subtitle.clone(),
                        reference: self.work.reference.clone(),
                        edition: self.work.edition,
                        imprint_id: self.work.imprint.imprint_id,
                        doi: self.work.doi.clone(),
                        publication_date: self.work.publication_date.clone(),
                        withdrawn_date: self.work.withdrawn_date.clone(),
                        place: self.work.place.clone(),
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
                        bibliography_note: self.work.bibliography_note.clone(),
                        toc: self.work.toc.clone(),
                        cover_url: self.work.cover_url.clone(),
                        cover_caption: self.work.cover_caption.clone(),
                        first_page: self.work.first_page.clone(),
                        last_page: self.work.last_page.clone(),
                        page_interval: self.work.page_interval.clone(),
                    },
                    ..Default::default()
                };
                let request = UpdateWorkRequest { body };
                self.push_work = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_work.fetch(Msg::SetWorkPushState));
                ctx.link()
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
                            ctx.link().history().unwrap().push(AdminRoute::Works);
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
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::DeleteWork => {
                let body = DeleteWorkRequestBody {
                    variables: DeleteVariables {
                        work_id: self.work.work_id,
                    },
                    ..Default::default()
                };
                let request = DeleteWorkRequest { body };
                self.delete_work = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_work.fetch(Msg::SetWorkDeleteState));
                ctx.link()
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
                if self.work.subtitle.neq_assign(value.to_opt_string()) {
                    self.work.full_title = self.work.compile_fulltitle();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeWorkType(work_type) => self.work.work_type.neq_assign(work_type),
            Msg::ChangeWorkStatus(work_status) => self.work.work_status.neq_assign(work_status),
            Msg::ChangeReference(value) => self.work.reference.neq_assign(value.to_opt_string()),
            Msg::ChangeImprint(imprint_id) => {
                // we already have the full list of imprints
                if let Some(imprint) = self
                    .data
                    .imprints
                    .iter()
                    .find(|i| i.imprint_id == imprint_id)
                {
                    self.work.imprint.neq_assign(imprint.clone())
                } else if imprint_id.is_nil() {
                    // ID may be nil if placeholder option was selected.
                    // If the work has any issues, self.work.imprint will be used
                    // as the basis of the dropdown list, so do not update it.
                    // GUI restrictions will prevent save while placeholder option is selected.
                    false
                } else {
                    // Imprint not found: clear existing selection
                    self.work.imprint.neq_assign(Default::default())
                }
            }
            Msg::ChangeEdition(edition) => self.work.edition.neq_assign(edition.to_opt_int()),
            Msg::ChangeDoi(value) => {
                if self.doi.neq_assign(value.trim().to_owned()) {
                    // If DOI is not correctly formatted, display a warning.
                    // Don't update self.work.doi yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.doi_warning.clear();
                    match self.doi.parse::<Doi>() {
                        Err(e) => {
                            match e {
                                // If no DOI was provided, no warning is required.
                                ThothError::DoiEmptyError => {}
                                _ => self.doi_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.doi = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeDate(value) => self.work.publication_date.neq_assign(value.to_opt_string()),
            Msg::ChangeWithdrawnDate(value) => {
                self.work.withdrawn_date.neq_assign(value.to_opt_string())
            }
            Msg::ChangePlace(value) => self.work.place.neq_assign(value.to_opt_string()),
            Msg::ChangePageCount(value) => self.work.page_count.neq_assign(value.to_opt_int()),
            Msg::ChangePageBreakdown(value) => {
                self.work.page_breakdown.neq_assign(value.to_opt_string())
            }
            Msg::ChangeFirstPage(value) => {
                if self.work.first_page.neq_assign(value.to_opt_string()) {
                    self.work.page_interval = self.work.compile_page_interval();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeLastPage(value) => {
                if self.work.last_page.neq_assign(value.to_opt_string()) {
                    self.work.page_interval = self.work.compile_page_interval();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeImageCount(value) => self.work.image_count.neq_assign(value.to_opt_int()),
            Msg::ChangeTableCount(value) => self.work.table_count.neq_assign(value.to_opt_int()),
            Msg::ChangeAudioCount(value) => self.work.audio_count.neq_assign(value.to_opt_int()),
            Msg::ChangeVideoCount(value) => self.work.video_count.neq_assign(value.to_opt_int()),
            Msg::ChangeLicense(value) => self.work.license.neq_assign(value.to_opt_string()),
            Msg::ChangeCopyright(copyright) => self
                .work
                .copyright_holder
                .neq_assign(copyright.to_opt_string()),
            Msg::ChangeLandingPage(value) => {
                self.work.landing_page.neq_assign(value.to_opt_string())
            }
            Msg::ChangeLccn(value) => self.work.lccn.neq_assign(value.to_opt_string()),
            Msg::ChangeOclc(value) => self.work.oclc.neq_assign(value.to_opt_string()),
            Msg::ChangeShortAbstract(value) => {
                self.work.short_abstract.neq_assign(value.to_opt_string())
            }
            Msg::ChangeLongAbstract(value) => {
                self.work.long_abstract.neq_assign(value.to_opt_string())
            }
            Msg::ChangeNote(value) => self.work.general_note.neq_assign(value.to_opt_string()),
            Msg::ChangeBibliographyNote(value) => self
                .work
                .bibliography_note
                .neq_assign(value.to_opt_string()),
            Msg::ChangeToc(value) => self.work.toc.neq_assign(value.to_opt_string()),
            Msg::ChangeCoverUrl(value) => self.work.cover_url.neq_assign(value.to_opt_string()),
            Msg::ChangeCoverCaption(value) => {
                self.work.cover_caption.neq_assign(value.to_opt_string())
            }
            Msg::UpdateRelatedWorks(related_works) => self.work.relations.neq_assign(related_works),
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
            Msg::UpdateReferences(references) => self.work.references.neq_assign(references),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access.clone());
        let updated_work = self.work_id.neq_assign(ctx.props().work_id);
        if updated_permissions || updated_work {
            // Required in order to retrieve updated list of imprints for dropdown
            // and/or full work if we have navigated direct from another Work page.
            ctx.link().send_message(Msg::GetWork);
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.fetch_work.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = ctx.link().callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::UpdateWork
                });
                // FormImprintSelect: while the work has any related issues, the imprint cannot
                // be changed, because an issue's series and work must both have the same imprint.
                let imprints = match self.work.issues.as_ref().unwrap_or(&vec![]).is_empty() {
                    true => self.data.imprints.clone(),
                    false => vec![self.work.imprint.clone()],
                };
                // FormWorkTypeSelect: while the work has any publications with ISBNs,
                // the work type cannot be changed to Book Chapter.
                let deactivated_types = match self
                    .work
                    .publications
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .any(|p| p.isbn.is_some())
                {
                    true => vec![WorkType::BookChapter],
                    false => vec![],
                };
                // Grey out chapter-specific or "book"-specific fields
                // based on currently selected work type.
                let is_chapter = self.work.work_type == WorkType::BookChapter;
                let is_not_withdrawn_or_superseded = self.work.work_status != WorkStatus::Withdrawn
                    && self.work.work_status != WorkStatus::Superseded;
                let is_active_withdrawn_or_superseded = self.work.work_status == WorkStatus::Active
                    || self.work.work_status == WorkStatus::Withdrawn
                    || self.work.work_status == WorkStatus::Superseded;
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
                                    <ConfirmDeleteComponent
                                        onclick={ ctx.link().callback(|_| Msg::DeleteWork) }
                                        object_name={ self.work.title.clone() }
                                    />
                                </p>
                            </div>
                        </nav>

                        <form onsubmit={ callback }>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormWorkTypeSelect
                                        label = "Work Type"
                                        value={ self.work.work_type }
                                        data={ self.data.work_types.clone() }
                                        deactivate={ deactivated_types.clone() }
                                        onchange={ ctx.link().callback(|e: Event|
                                            Msg::ChangeWorkType(WorkType::from_str(&e.to_value()).unwrap())
                                        ) }
                                        required = true
                                    />
                                    <FormWorkStatusSelect
                                        label = "Work Status"
                                        value={ self.work.work_status }
                                        data={ self.data.work_statuses.clone() }
                                        onchange={ ctx.link().callback(|e: Event|
                                            Msg::ChangeWorkStatus(WorkStatus::from_str(&e.to_value()).unwrap())
                                        ) }
                                        required = true
                                    />
                                    <FormImprintSelect
                                        label = "Imprint"
                                        value={ self.work.imprint.imprint_id }
                                        data={ imprints.clone() }
                                        onchange={ ctx.link().callback(|e: Event|
                                            Msg::ChangeImprint(Uuid::parse_str(&e.to_value()).unwrap_or_default())
                                        ) }
                                        required = true
                                    />
                                </div>
                            </div>
                            <FormTextInput
                                label = "Title"
                                value={ self.work.title.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeTitle(e.to_value())) }
                                required = true
                            />
                            <FormTextInput
                                label = "Subtitle"
                                value={ self.work.subtitle.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeSubtitle(e.to_value())) }
                            />
                            <FormNumberInput
                                label = "Edition"
                                value={ self.work.edition }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeEdition(e.to_value())) }
                                required = true
                                min={ "1".to_string() }
                                deactivated={ is_chapter }
                            />
                            <FormDateInput
                                label = "Publication Date"
                                value={ self.work.publication_date.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeDate(e.to_value())) }
                                required = { is_active_withdrawn_or_superseded }
                            />
                            <FormDateInput
                                label = "Withdrawn Date"
                                value={ self.work.withdrawn_date.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeWithdrawnDate(e.to_value())) }
                                required ={ !is_not_withdrawn_or_superseded }
                                deactivated={ is_not_withdrawn_or_superseded }
                                />
                            <FormTextInput
                                label = "Place of Publication"
                                value={ self.work.place.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePlace(e.to_value())) }
                            />
                            <div class="field">
                                <div class="tile is-ancestor">
                                    <div class="tile is-2 is-parent">
                                        <div class="tile is-child">
                                            <figure class="image is-fullwidth">
                                                <img
                                                    src={self.work.cover_url.clone().unwrap_or_default().clone()}
                                                    loading="lazy"
                                                />
                                            </figure>
                                        </div>
                                    </div>
                                    <div class="tile is-parent">
                                        <div class="tile is-child">
                                            <FormUrlInput
                                                label = "Cover URL"
                                                value={ self.work.cover_url.clone() }
                                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeCoverUrl(e.to_value())) }
                                            />
                                            <FormTextarea
                                                label = "Cover Caption"
                                                value={ self.work.cover_caption.clone() }
                                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeCoverCaption(e.to_value())) }
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormTextInputExtended
                                        label = "DOI"
                                        statictext={ DOI_DOMAIN }
                                        value={ self.doi.clone() }
                                        tooltip={ self.doi_warning.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeDoi(e.to_value())) }
                                    />
                                    <FormTextInput
                                        label = "LCCN"
                                        value={ self.work.lccn.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLccn(e.to_value())) }
                                        deactivated={ is_chapter }
                                    />
                                    <FormTextInput
                                        label = "OCLC Number"
                                        value={ self.work.oclc.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOclc(e.to_value())) }
                                        deactivated={ is_chapter }
                                    />
                                    <FormTextInput
                                        label = "Internal Reference"
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeReference(e.to_value())) }
                                        value={ self.work.reference.clone() }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormNumberInput
                                        label = "Page Count"
                                        value={ self.work.page_count }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePageCount(e.to_value())) }
                                    />
                                    <FormTextInput
                                        label = "Page Breakdown"
                                        value={ self.work.page_breakdown.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePageBreakdown(e.to_value())) }
                                    />
                                    <FormTextInput
                                        label = "First Page"
                                        value={ self.work.first_page.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFirstPage(e.to_value())) }
                                        deactivated={ !is_chapter }
                                    />
                                    <FormTextInput
                                        label = "Last Page"
                                        value={ self.work.last_page.clone() }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLastPage(e.to_value())) }
                                        deactivated={ !is_chapter }
                                    />
                                </div>
                            </div>
                            <div class="field is-horizontal">
                                <div class="field-body">
                                    <FormNumberInput
                                        label = "Image Count"
                                        value={ self.work.image_count }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeImageCount(e.to_value())) }
                                    />
                                    <FormNumberInput
                                        label = "Table Count"
                                        value={ self.work.table_count }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeTableCount(e.to_value())) }
                                    />
                                    <FormNumberInput
                                        label = "Audio Count"
                                        value={ self.work.audio_count }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeAudioCount(e.to_value())) }
                                    />
                                    <FormNumberInput
                                        label = "Video Count"
                                        value={ self.work.video_count }
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeVideoCount(e.to_value())) }
                                    />
                                </div>
                            </div>
                            <FormUrlInput
                                label = "License"
                                value={ self.work.license.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLicense(e.to_value())) }
                            />
                            <FormTextInput
                                label = "Copyright Holder"
                                value={ self.work.copyright_holder.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeCopyright(e.to_value())) }
                            />
                            <FormUrlInput
                                label = "Landing Page"
                                value={ self.work.landing_page.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLandingPage(e.to_value())) }
                            />
                            <FormTextarea
                                label = "Short Abstract"
                                value={ self.work.short_abstract.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeShortAbstract(e.to_value())) }
                            />
                            <FormTextarea
                                label = "Long Abstract"
                                value={ self.work.long_abstract.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLongAbstract(e.to_value())) }
                            />
                            <FormTextarea
                                label = "General Note"
                                value={ self.work.general_note.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeNote(e.to_value())) }
                            />
                            <FormTextarea
                                label = "Bibliography Note"
                                value={ self.work.bibliography_note.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeBibliographyNote(e.to_value())) }
                            />
                            <FormTextarea
                                label = "Table of Content"
                                value={ self.work.toc.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeToc(e.to_value())) }
                                deactivated={ is_chapter }
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
                                { RELATIONS_INFO }
                            </div>
                        </article>

                        <RelatedWorksFormComponent
                            relations={ self.work.relations.clone() }
                            work_id={ self.work.work_id }
                            current_user={ ctx.props().current_user.clone() }
                            update_relations={ ctx.link().callback(Msg::UpdateRelatedWorks) }
                        />
                        {
                            if self.work_type == WorkType::Monograph
                                || self.work_type == WorkType::EditedBook
                                || self.work_type == WorkType::Textbook {
                                html! {
                                    // Convenience button for adding Chapter relations with inherited values
                                    <NewChapterComponent
                                        work={ self.work.clone() }
                                        relations={ self.work.relations.clone() }
                                        update_relations={ ctx.link().callback(Msg::UpdateRelatedWorks) }
                                    />
                                }
                            } else {
                                html! {}
                            }
                        }
                        <ContributionsFormComponent
                            contributions={ self.work.contributions.clone() }
                            work_id={ self.work.work_id }
                            update_contributions={ ctx.link().callback(Msg::UpdateContributions) }
                        />
                        <PublicationsFormComponent
                            publications={ self.work.publications.clone() }
                            work_id={ self.work.work_id }
                            work_type={ self.work_type }
                            update_publications={ ctx.link().callback(Msg::UpdatePublications) }
                        />
                        <LanguagesFormComponent
                            languages={ self.work.languages.clone() }
                            work_id={ self.work.work_id }
                            update_languages={ ctx.link().callback(Msg::UpdateLanguages) }
                        />
                        <SubjectsFormComponent
                            subjects={ self.work.subjects.clone() }
                            work_id={ self.work.work_id }
                            update_subjects={ ctx.link().callback(Msg::UpdateSubjects) }
                        />
                        <IssuesFormComponent
                            issues={ self.work.issues.clone() }
                            work_id={ self.work.work_id }
                            imprint_id={ self.imprint_id }
                            current_user={ ctx.props().current_user.clone() }
                            update_issues={ ctx.link().callback(Msg::UpdateIssues) }
                        />
                        <FundingsFormComponent
                            fundings={ self.work.fundings.clone() }
                            work_id={ self.work.work_id }
                            update_fundings={ ctx.link().callback(Msg::UpdateFundings) }
                        />
                        <ReferencesFormComponent
                            references={ self.work.references.clone() }
                            work_id={ self.work.work_id }
                            update_references={ ctx.link().callback(Msg::UpdateReferences) }
                        />
                    </>
                }
            }
            FetchState::Failed(_, err) => html! {
                { ThothError::from(err).to_string() }
            },
        }
    }
}
