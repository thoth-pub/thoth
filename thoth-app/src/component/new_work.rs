use std::str::FromStr;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::work::WorkStatus;
use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use thoth_api::model::{Doi, DOI_DOMAIN};
use thoth_errors::ThothError;
use uuid::Uuid;
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
use crate::component::utils::FormDateInput;
use crate::component::utils::FormImprintSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextInputExtended;
use crate::component::utils::FormTextarea;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormWorkStatusSelect;
use crate::component::utils::FormWorkTypeSelect;
use crate::models::imprint::imprints_query::FetchActionImprints;
use crate::models::imprint::imprints_query::FetchImprints;
use crate::models::imprint::imprints_query::ImprintsRequest;
use crate::models::imprint::imprints_query::ImprintsRequestBody;
use crate::models::imprint::imprints_query::Variables as ImprintsVariables;
use crate::models::work::create_work_mutation::CreateWorkRequest;
use crate::models::work::create_work_mutation::CreateWorkRequestBody;
use crate::models::work::create_work_mutation::PushActionCreateWork;
use crate::models::work::create_work_mutation::PushCreateWork;
use crate::models::work::create_work_mutation::Variables;
use crate::models::work::work_statuses_query::FetchActionWorkStatuses;
use crate::models::work::work_statuses_query::FetchWorkStatuses;
use crate::models::work::work_types_query::FetchActionWorkTypes;
use crate::models::work::work_types_query::FetchWorkTypes;
use crate::models::work::WorkStatusValues;
use crate::models::work::WorkTypeValues;
use crate::models::EditRoute;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

use super::ToOption;

pub struct NewWorkComponent {
    work: WorkWithRelations,
    // Track the user-entered DOI string, which may not be validly formatted
    doi: String,
    doi_warning: String,
    // Track imprint stored in database, as distinct from imprint selected in dropdown
    imprint_id: Uuid,
    push_work: PushCreateWork,
    data: WorkFormData,
    fetch_imprints: FetchImprints,
    fetch_work_types: FetchWorkTypes,
    fetch_work_statuses: FetchWorkStatuses,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
    props: Props,
}

#[derive(Default)]
struct WorkFormData {
    imprints: Vec<ImprintWithPublisher>,
    work_types: Vec<WorkTypeValues>,
    work_statuses: Vec<WorkStatusValues>,
}

#[allow(clippy::large_enum_variant)]
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
    ChangeImprint(Uuid),
    ChangeEdition(String),
    ChangeDoi(String),
    ChangeDate(String),
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
    ChangeToc(String),
    ChangeCoverUrl(String),
    ChangeCoverCaption(String),
    ChangeRoute(AppRoute),
}
#[derive(Clone, Properties)]
pub struct Props {
    pub current_user: AccountDetails,
    pub previous_route: AdminRoute,
}

impl Component for NewWorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_work = Default::default();
        let router = RouteAgentDispatcher::new();
        let notification_bus = NotificationBus::dispatcher();
        let work = WorkWithRelations {
            work_type: match props.previous_route {
                AdminRoute::Chapters => WorkType::BookChapter,
                _ => Default::default(),
            },
            edition: match props.previous_route {
                AdminRoute::Chapters => Default::default(),
                _ => Some(1),
            },
            ..Default::default()
        };
        let doi = Default::default();
        let doi_warning = Default::default();
        let imprint_id: Uuid = Default::default();
        let data: WorkFormData = Default::default();
        let fetch_imprints: FetchImprints = Default::default();
        let fetch_work_types: FetchWorkTypes = Default::default();
        let fetch_work_statuses: FetchWorkStatuses = Default::default();

        link.send_message(Msg::GetImprints);
        link.send_message(Msg::GetWorkTypes);
        link.send_message(Msg::GetWorkStatuses);

        NewWorkComponent {
            work,
            doi,
            doi_warning,
            imprint_id,
            push_work,
            data,
            fetch_imprints,
            fetch_work_types,
            fetch_work_statuses,
            link,
            router,
            notification_bus,
            props,
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
                let body = ImprintsRequestBody {
                    variables: ImprintsVariables {
                        publishers: self.props.current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = ImprintsRequest { body };
                self.fetch_imprints = Fetch::new(request);

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
                            self.link.send_message(Msg::ChangeRoute(w.edit_route()));
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
                // Only update the DOI value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
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
                        imprint_id: self.imprint_id,
                        first_page: self.work.first_page.clone(),
                        last_page: self.work.last_page.clone(),
                        page_interval: self.work.page_interval.clone(),
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
            Msg::ChangeImprint(imprint_id) => self.imprint_id.neq_assign(imprint_id),
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
                .neq_assign(copyright.trim().to_owned()),
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
            Msg::ChangeToc(value) => self.work.toc.neq_assign(value.to_opt_string()),
            Msg::ChangeCoverUrl(value) => self.work.cover_url.neq_assign(value.to_opt_string()),
            Msg::ChangeCoverCaption(value) => {
                self.work.cover_caption.neq_assign(value.to_opt_string())
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let updated_permissions =
            self.props.current_user.resource_access != props.current_user.resource_access;
        self.props = props;
        if updated_permissions {
            self.link.send_message(Msg::GetImprints);
        }
        false
    }

    fn view(&self) -> Html {
        let callback = self.link.callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateWork
        });
        // Grey out chapter-specific or "book"-specific fields
        // based on currently selected work type.
        let is_chapter = self.work.work_type == WorkType::BookChapter;
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
                                value=self.work.work_type.clone()
                                data=self.data.work_types.clone()
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
                                value=self.work.work_status.clone()
                                data=self.data.work_statuses.clone()
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
                                value=self.imprint_id
                                data=self.data.imprints.clone()
                                onchange=self.link.callback(|event| match event {
                                    ChangeData::Select(elem) => {
                                        let value = elem.value();
                                        Msg::ChangeImprint(Uuid::parse_str(&value).unwrap_or_default())
                                    }
                                    _ => unreachable!(),
                                })
                                required = true
                            />
                        </div>
                    </div>
                    <FormTextInput
                        label = "Title"
                        value=self.work.title.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeTitle(e.value))
                        required = true
                    />
                    <FormTextInput
                        label = "Subtitle"
                        value=self.work.subtitle.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeSubtitle(e.value))
                    />
                    <FormNumberInput
                        label = "Edition"
                        value=self.work.edition
                        oninput=self.link.callback(|e: InputData| Msg::ChangeEdition(e.value))
                        required = true
                        min = "1".to_string()
                        deactivated = is_chapter
                    />
                    <FormDateInput
                        label = "Publication Date"
                        value=self.work.publication_date.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeDate(e.value))
                    />
                    <FormTextInput
                        label = "Place of Publication"
                        value=self.work.place.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangePlace(e.value))
                    />
                    <div class="field">
                        <div class="tile is-ancestor">
                            <div class="tile is-2 is-parent">
                                <div class="tile is-child">
                                    <figure class="image is-fullwidth">
                                        <img
                                            src={self.work.cover_url.clone().unwrap_or_else(|| "".to_string()).clone()}
                                            loading="lazy"
                                        />
                                    </figure>
                                </div>
                            </div>
                            <div class="tile is-parent">
                                <div class="tile is-child">
                                    <FormUrlInput
                                        label = "Cover URL"
                                        value=self.work.cover_url.clone()
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeCoverUrl(e.value))
                                    />
                                    <FormTextarea
                                        label = "Cover Caption"
                                        value=self.work.cover_caption.clone()
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeCoverCaption(e.value))
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="field is-horizontal">
                        <div class="field-body">
                            <FormTextInputExtended
                                label = "DOI"
                                statictext = DOI_DOMAIN
                                value=self.doi.clone()
                                tooltip=self.doi_warning.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeDoi(e.value))
                            />
                            <FormTextInput
                                label = "LCCN"
                                value=self.work.lccn.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeLccn(e.value))
                                deactivated = is_chapter
                            />
                            <FormTextInput
                                label = "OCLC Number"
                                value=self.work.oclc.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeOclc(e.value))
                                deactivated = is_chapter
                            />
                            <FormTextInput
                                label = "Internal Reference"
                                oninput=self.link.callback(|e: InputData| Msg::ChangeReference(e.value))
                                value=self.work.reference.clone()
                            />
                        </div>
                    </div>
                    <div class="field is-horizontal">
                        <div class="field-body">
                            <FormNumberInput
                                label = "Page Count"
                                value=self.work.page_count
                                oninput=self.link.callback(|e: InputData| Msg::ChangePageCount(e.value))
                            />
                            <FormTextInput
                                label = "Page Breakdown"
                                value=self.work.page_breakdown.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangePageBreakdown(e.value))
                            />
                            <FormTextInput
                                label = "First Page"
                                value=self.work.first_page.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeFirstPage(e.value))
                                deactivated = !is_chapter
                            />
                            <FormTextInput
                                label = "Last Page"
                                value=self.work.last_page.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeLastPage(e.value))
                                deactivated = !is_chapter
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
                        label = "License"
                        value=self.work.license.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLicense(e.value))
                    />
                    <FormTextInput
                        label = "Copyright Holder"
                        value=self.work.copyright_holder.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeCopyright(e.value))
                        required = true
                    />
                    <FormUrlInput
                        label = "Landing Page"
                        value=self.work.landing_page.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLandingPage(e.value))
                    />
                    <FormTextarea
                        label = "Short Abstract"
                        value=self.work.short_abstract.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeShortAbstract(e.value))
                    />
                    <FormTextarea
                        label = "Long Abstract"
                        value=self.work.long_abstract.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLongAbstract(e.value))
                    />
                    <FormTextarea
                        label = "General Note"
                        value=self.work.general_note.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeNote(e.value))
                    />
                    <FormTextarea
                        label = "Table of Content"
                        value=self.work.toc.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeToc(e.value))
                        deactivated = is_chapter
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
