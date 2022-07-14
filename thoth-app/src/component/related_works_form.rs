use std::str::FromStr;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::work::Work;
use thoth_api::model::work_relation::RelationType;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
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
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormRelationTypeSelect;
use crate::models::work::slim_works_query::FetchActionSlimWorks;
use crate::models::work::slim_works_query::FetchSlimWorks;
use crate::models::work::slim_works_query::SlimWorksRequest;
use crate::models::work::slim_works_query::SlimWorksRequestBody;
use crate::models::work::slim_works_query::Variables;
use crate::models::work_relation::create_work_relation_mutation::CreateWorkRelationRequest;
use crate::models::work_relation::create_work_relation_mutation::CreateWorkRelationRequestBody;
use crate::models::work_relation::create_work_relation_mutation::PushActionCreateWorkRelation;
use crate::models::work_relation::create_work_relation_mutation::PushCreateWorkRelation;
use crate::models::work_relation::create_work_relation_mutation::Variables as CreateVariables;
use crate::models::work_relation::delete_work_relation_mutation::DeleteWorkRelationRequest;
use crate::models::work_relation::delete_work_relation_mutation::DeleteWorkRelationRequestBody;
use crate::models::work_relation::delete_work_relation_mutation::PushActionDeleteWorkRelation;
use crate::models::work_relation::delete_work_relation_mutation::PushDeleteWorkRelation;
use crate::models::work_relation::delete_work_relation_mutation::Variables as DeleteVariables;
use crate::models::work_relation::relation_types_query::FetchActionRelationTypes;
use crate::models::work_relation::relation_types_query::FetchRelationTypes;
use crate::models::work_relation::update_work_relation_mutation::PushActionUpdateWorkRelation;
use crate::models::work_relation::update_work_relation_mutation::PushUpdateWorkRelation;
use crate::models::work_relation::update_work_relation_mutation::UpdateWorkRelationRequest;
use crate::models::work_relation::update_work_relation_mutation::UpdateWorkRelationRequestBody;
use crate::models::work_relation::update_work_relation_mutation::Variables as UpdateVariables;
use crate::models::work_relation::RelationTypeValues;
use crate::models::Dropdown;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_RELATIONS;
use crate::string::REMOVE_BUTTON;
use crate::string::VIEW_BUTTON;

pub struct RelatedWorksFormComponent {
    props: Props,
    data: RelatedWorksFormData,
    relation: WorkRelationWithRelatedWork,
    show_modal_form: bool,
    in_edit_mode: bool,
    show_results: bool,
    fetch_works: FetchSlimWorks,
    fetch_relation_types: FetchRelationTypes,
    create_relation: PushCreateWorkRelation,
    delete_relation: PushDeleteWorkRelation,
    update_relation: PushUpdateWorkRelation,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
}

#[derive(Default)]
struct RelatedWorksFormData {
    works: Vec<Work>,
    relation_types: Vec<RelationTypeValues>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleModalFormDisplay(bool, Option<WorkRelationWithRelatedWork>),
    SetWorksFetchState(FetchActionSlimWorks),
    GetWorks,
    SetRelationTypesFetchState(FetchActionRelationTypes),
    GetRelationTypes,
    ToggleSearchResultDisplay(bool),
    SearchWork(String),
    SetRelationCreateState(PushActionCreateWorkRelation),
    CreateWorkRelation,
    SetRelationUpdateState(PushActionUpdateWorkRelation),
    UpdateWorkRelation,
    SetRelationDeleteState(PushActionDeleteWorkRelation),
    DeleteWorkRelation(Uuid),
    AddRelation(Work),
    ChangeRelationtype(RelationType),
    ChangeOrdinal(String),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub relations: Option<Vec<WorkRelationWithRelatedWork>>,
    pub work_id: Uuid,
    pub current_user: AccountDetails,
    pub update_relations: Callback<Option<Vec<WorkRelationWithRelatedWork>>>,
}

impl Component for RelatedWorksFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: RelatedWorksFormData = Default::default();
        let relation: WorkRelationWithRelatedWork = Default::default();
        let show_modal_form = false;
        let in_edit_mode = false;
        let show_results = false;
        let body = SlimWorksRequestBody {
            variables: Variables {
                publishers: props.current_user.resource_access.restricted_to(),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = SlimWorksRequest { body };
        let fetch_works = Fetch::new(request);
        let fetch_relation_types = Default::default();
        let create_relation = Default::default();
        let delete_relation = Default::default();
        let update_relation = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let router = RouteAgentDispatcher::new();

        link.send_message(Msg::GetWorks);
        link.send_message(Msg::GetRelationTypes);

        RelatedWorksFormComponent {
            props,
            data,
            relation,
            show_modal_form,
            in_edit_mode,
            show_results,
            fetch_works,
            fetch_relation_types,
            create_relation,
            delete_relation,
            update_relation,
            link,
            notification_bus,
            router,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, r) => {
                self.show_modal_form = show_form;
                self.in_edit_mode = r.is_some();
                if show_form {
                    if let Some(relation) = r {
                        // Editing existing relation: load its current values.
                        self.relation = relation;
                    }
                }
                true
            }
            Msg::SetWorksFetchState(fetch_state) => {
                self.fetch_works.apply(fetch_state);
                self.data.works = match self.fetch_works.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.works.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetWorks => {
                self.link
                    .send_future(self.fetch_works.fetch(Msg::SetWorksFetchState));
                self.link
                    .send_message(Msg::SetWorksFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetRelationTypesFetchState(fetch_state) => {
                self.fetch_relation_types.apply(fetch_state);
                self.data.relation_types = match self.fetch_relation_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.relation_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetRelationTypes => {
                self.link.send_future(
                    self.fetch_relation_types
                        .fetch(Msg::SetRelationTypesFetchState),
                );
                self.link
                    .send_message(Msg::SetRelationTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetRelationCreateState(fetch_state) => {
                self.create_relation.apply(fetch_state);
                match self.create_relation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_work_relation {
                        Some(r) => {
                            let relation = r.clone();
                            let mut relations: Vec<WorkRelationWithRelatedWork> =
                                self.props.relations.clone().unwrap_or_default();
                            relations.push(relation);
                            self.props.update_relations.emit(Some(relations));
                            self.link
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            self.link
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateWorkRelation => {
                let body = CreateWorkRelationRequestBody {
                    variables: CreateVariables {
                        relator_work_id: self.props.work_id,
                        related_work_id: self.relation.related_work_id,
                        relation_type: self.relation.relation_type,
                        relation_ordinal: self.relation.relation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateWorkRelationRequest { body };
                self.create_relation = Fetch::new(request);
                self.link
                    .send_future(self.create_relation.fetch(Msg::SetRelationCreateState));
                self.link
                    .send_message(Msg::SetRelationCreateState(FetchAction::Fetching));
                false
            }
            Msg::SetRelationUpdateState(fetch_state) => {
                self.update_relation.apply(fetch_state);
                match self.update_relation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_work_relation {
                        Some(r) => {
                            let mut relations: Vec<WorkRelationWithRelatedWork> =
                                self.props.relations.clone().unwrap_or_default();
                            if let Some(relation) = relations
                                .iter_mut()
                                .find(|rn| rn.work_relation_id == r.work_relation_id)
                            {
                                *relation = r.clone();
                                self.props.update_relations.emit(Some(relations));
                            } else {
                                // This should not be possible: the updated relation returned from the
                                // database does not match any of the locally-stored relation data.
                                // Refreshing the page will reload the local data from the database.
                                self.notification_bus.send(Request::NotificationBusMsg((
                                    "Changes were saved but display failed to update. Refresh your browser to view current data.".to_string(),
                                    NotificationStatus::Warning,
                                )));
                            }
                            self.link
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            self.link
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateWorkRelation => {
                let body = UpdateWorkRelationRequestBody {
                    variables: UpdateVariables {
                        work_relation_id: self.relation.work_relation_id,
                        relator_work_id: self.props.work_id,
                        related_work_id: self.relation.related_work_id,
                        relation_type: self.relation.relation_type,
                        relation_ordinal: self.relation.relation_ordinal,
                    },
                    ..Default::default()
                };
                let request = UpdateWorkRelationRequest { body };
                self.update_relation = Fetch::new(request);
                self.link
                    .send_future(self.update_relation.fetch(Msg::SetRelationUpdateState));
                self.link
                    .send_message(Msg::SetRelationUpdateState(FetchAction::Fetching));
                false
            }
            Msg::SetRelationDeleteState(fetch_state) => {
                self.delete_relation.apply(fetch_state);
                match self.delete_relation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_work_relation {
                        Some(relation) => {
                            let to_keep: Vec<WorkRelationWithRelatedWork> = self
                                .props
                                .relations
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|r| r.work_relation_id != relation.work_relation_id)
                                .collect();
                            self.props.update_relations.emit(Some(to_keep));
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
            Msg::DeleteWorkRelation(work_relation_id) => {
                let body = DeleteWorkRelationRequestBody {
                    variables: DeleteVariables { work_relation_id },
                    ..Default::default()
                };
                let request = DeleteWorkRelationRequest { body };
                self.delete_relation = Fetch::new(request);
                self.link
                    .send_future(self.delete_relation.fetch(Msg::SetRelationDeleteState));
                self.link
                    .send_message(Msg::SetRelationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddRelation(work) => {
                self.relation.related_work_id = work.work_id;
                self.relation.related_work = work;
                self.link
                    .send_message(Msg::ToggleModalFormDisplay(true, None));
                true
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchWork(value) => {
                let body = SlimWorksRequestBody {
                    variables: Variables {
                        filter: Some(value),
                        limit: Some(9999),
                        publishers: self.props.current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = SlimWorksRequest { body };
                self.fetch_works = Fetch::new(request);
                self.link.send_message(Msg::GetWorks);
                false
            }
            Msg::ChangeRelationtype(val) => self.relation.relation_type.neq_assign(val),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.relation.relation_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let relations = self.props.relations.clone().unwrap_or_default();
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false, None)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Related Works" }
                </p>
                <div class="panel-block">
                    <div class=self.search_dropdown_status() style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Work"
                                        aria-haspopup="true"
                                        aria-controls="works-menu"
                                        oninput=self.link.callback(|e: InputData| Msg::SearchWork(e.value))
                                        onfocus=self.link.callback(|_| Msg::ToggleSearchResultDisplay(true))
                                        onblur=self.link.callback(|_| Msg::ToggleSearchResultDisplay(false))
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                            </div>
                        </div>
                        <div class="dropdown-menu" id="works-menu" role="menu">
                            <div class="dropdown-content">
                                {
                                    for self.data.works.iter().map(|w| {
                                        let work = w.clone();
                                        // avoid listing works where a relation already exists
                                        if self.props.relations
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .any(|r| r.related_work_id == work.work_id)
                                            // avoid listing current work
                                            || self.props.work_id == work.work_id {
                                            html! {}
                                        } else {
                                            w.as_dropdown_item(
                                                self.link.callback(move |_| {
                                                    Msg::AddRelation(work.clone())
                                                })
                                            )
                                        }
                                    })
                                }
                            </div>
                        </div>
                    </div>
                </div>
                <div class=self.modal_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ self.modal_form_title() }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="relations-form" onsubmit={ self.modal_form_action() }>
                                <FormRelationTypeSelect
                                    label = "Relation Type"
                                    value=self.relation.relation_type
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeRelationtype(RelationType::from_str(&value).unwrap())
                                        }
                                        _ => unreachable!(),
                                    })
                                    data=self.data.relation_types.clone()
                                    required = true
                                />
                                <div class="field">
                                    <label class="label">{ "Work" }</label>
                                    <div class="control is-expanded">
                                        {&self.relation.related_work.full_title}
                                    </div>
                                </div>
                                <FormNumberInput
                                    label = "Relation Ordinal"
                                    value=self.relation.relation_ordinal
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeOrdinal(e.value))
                                    required = true
                                    min = "1".to_string()
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="relations-form"
                            >
                                { self.modal_form_button() }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !relations.is_empty() {
                        html!{{for relations.iter().map(|r| self.render_relation(r))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_RELATIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl RelatedWorksFormComponent {
    fn modal_form_status(&self) -> String {
        match self.show_modal_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn modal_form_title(&self) -> String {
        match self.in_edit_mode {
            true => "Edit Related Work".to_string(),
            false => "New Related Work".to_string(),
        }
    }

    fn modal_form_button(&self) -> String {
        match self.in_edit_mode {
            true => "Save Related Work".to_string(),
            false => "Add Related Work".to_string(),
        }
    }

    fn modal_form_action(&self) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => self.link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateWorkRelation
            }),
            false => self.link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateWorkRelation
            }),
        }
    }

    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_relation(&self, r: &WorkRelationWithRelatedWork) -> Html {
        let relation = r.clone();
        let relation_id = r.work_relation_id;
        let route = r.related_work.edit_route();
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-user" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Relation Type" }</label>
                        <div class="control is-expanded">
                            {&r.relation_type}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Work" }</label>
                        <div class="control is-expanded">
                            {&r.related_work.full_title}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Relation Ordinal" }</label>
                        <div class="control is-expanded">
                            {&r.relation_ordinal.clone()}
                        </div>
                    </div>

                    <div class="field is-grouped is-grouped-right">
                        <div class="control">
                            <a
                                class="button is-info"
                                onclick=self.link.callback(move |_| Msg::ChangeRoute(route.clone()))
                            >
                                { VIEW_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-success"
                                onclick=self.link.callback(move |_| Msg::ToggleModalFormDisplay(true, Some(relation.clone())))
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::DeleteWorkRelation(relation_id))
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
