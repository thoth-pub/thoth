use std::str::FromStr;
use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::work::Work;
use thoth_api::model::work_relation::RelationType;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
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
use crate::models::work_relation::RelationTypeValues;
use crate::models::Dropdown;
use crate::models::EditRoute;
use crate::route::AdminRoute;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_RELATIONS;
use crate::string::REMOVE_BUTTON;
use crate::string::VIEW_BUTTON;

use super::ToElementValue;

pub struct RelatedWorksFormComponent {
    data: RelatedWorksFormData,
    new_relation: WorkRelationWithRelatedWork,
    show_add_form: bool,
    show_results: bool,
    fetch_works: FetchSlimWorks,
    fetch_relation_types: FetchRelationTypes,
    push_relation: PushCreateWorkRelation,
    delete_relation: PushDeleteWorkRelation,
    notification_bus: NotificationDispatcher,
    // Store props value locally in order to test whether it has been updated on props change
    resource_access: AccountAccess,
}

#[derive(Default)]
struct RelatedWorksFormData {
    works: Vec<Work>,
    relation_types: Vec<RelationTypeValues>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetWorksFetchState(FetchActionSlimWorks),
    GetWorks,
    SetRelationTypesFetchState(FetchActionRelationTypes),
    GetRelationTypes,
    ToggleSearchResultDisplay(bool),
    SearchWork(String),
    SetRelationPushState(PushActionCreateWorkRelation),
    CreateWorkRelation,
    SetRelationDeleteState(PushActionDeleteWorkRelation),
    DeleteWorkRelation(Uuid),
    AddRelation(Work),
    ChangeRelationtype(RelationType),
    ChangeOrdinal(String),
    ChangeRoute(AdminRoute),
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

    fn create(ctx: &Context<Self>) -> Self {
        let data: RelatedWorksFormData = Default::default();
        let new_relation: WorkRelationWithRelatedWork = Default::default();
        let show_add_form = false;
        let show_results = false;
        let body = SlimWorksRequestBody {
            variables: Variables {
                publishers: ctx.props().current_user.resource_access.restricted_to(),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = SlimWorksRequest { body };
        let fetch_works = Fetch::new(request);
        let fetch_relation_types = Default::default();
        let push_relation = Default::default();
        let delete_relation = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let resource_access = ctx.props().current_user.resource_access.clone();

        ctx.link().send_message(Msg::GetWorks);
        ctx.link().send_message(Msg::GetRelationTypes);

        RelatedWorksFormComponent {
            data,
            new_relation,
            show_add_form,
            show_results,
            fetch_works,
            fetch_relation_types,
            push_relation,
            delete_relation,
            notification_bus,
            resource_access,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
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
                ctx.link()
                    .send_future(self.fetch_works.fetch(Msg::SetWorksFetchState));
                ctx.link()
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
                ctx.link().send_future(
                    self.fetch_relation_types
                        .fetch(Msg::SetRelationTypesFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetRelationTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetRelationPushState(fetch_state) => {
                self.push_relation.apply(fetch_state);
                match self.push_relation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_work_relation {
                        Some(r) => {
                            let relation = r.clone();
                            let mut relations: Vec<WorkRelationWithRelatedWork> =
                                ctx.props().relations.clone().unwrap_or_default();
                            relations.push(relation);
                            ctx.props().update_relations.emit(Some(relations));
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
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
                        relator_work_id: ctx.props().work_id,
                        related_work_id: self.new_relation.related_work_id,
                        relation_type: self.new_relation.relation_type,
                        relation_ordinal: self.new_relation.relation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateWorkRelationRequest { body };
                self.push_relation = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_relation.fetch(Msg::SetRelationPushState));
                ctx.link()
                    .send_message(Msg::SetRelationPushState(FetchAction::Fetching));
                false
            }
            Msg::SetRelationDeleteState(fetch_state) => {
                self.delete_relation.apply(fetch_state);
                match self.delete_relation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_work_relation {
                        Some(relation) => {
                            let to_keep: Vec<WorkRelationWithRelatedWork> = ctx
                                .props()
                                .relations
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|r| r.work_relation_id != relation.work_relation_id)
                                .collect();
                            ctx.props().update_relations.emit(Some(to_keep));
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
                ctx.link()
                    .send_future(self.delete_relation.fetch(Msg::SetRelationDeleteState));
                ctx.link()
                    .send_message(Msg::SetRelationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddRelation(work) => {
                self.new_relation.related_work_id = work.work_id;
                self.new_relation.related_work = work;
                ctx.link().send_message(Msg::ToggleAddFormDisplay(true));
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
                        publishers: ctx.props().current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = SlimWorksRequest { body };
                self.fetch_works = Fetch::new(request);
                ctx.link().send_message(Msg::GetWorks);
                false
            }
            Msg::ChangeRelationtype(val) => self.new_relation.relation_type.neq_assign(val),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.new_relation.relation_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeRoute(r) => {
                ctx.link().history().unwrap().push(r);
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access.clone());
        if updated_permissions {
            // Reload works list to reflect the user's access rights.
            // This will override any search box filtering, but should only occur rarely.
            let body = SlimWorksRequestBody {
                variables: Variables {
                    publishers: ctx.props().current_user.resource_access.restricted_to(),
                    ..Default::default()
                },
                ..Default::default()
            };
            let request = SlimWorksRequest { body };
            self.fetch_works = Fetch::new(request);
            ctx.link().send_message(Msg::GetWorks);
            false
        } else {
            true
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let relations = ctx.props().relations.clone().unwrap_or_default();
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Related Works" }
                </p>
                <div class="panel-block">
                    <div class={ self.search_dropdown_status() } style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Work"
                                        aria-haspopup="true"
                                        aria-controls="works-menu"
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::SearchWork(e.to_value())) }
                                        onfocus={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(true)) }
                                        onblur={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(false)) }
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
                                        if ctx.props().relations
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .any(|r| r.related_work_id == work.work_id)
                                            // avoid listing current work
                                            || ctx.props().work_id == work.work_id {
                                            html! {}
                                        } else {
                                            w.as_dropdown_item(
                                                ctx.link().callback(move |_| {
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
                <div class={ self.add_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Related Work" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="relations-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateWorkRelation
                            }) }
                            >
                                <FormRelationTypeSelect
                                    label = "Relation Type"
                                    value={ self.new_relation.relation_type }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeRelationtype(RelationType::from_str(&e.to_value()).unwrap())
                                    ) }
                                    data={ self.data.relation_types.clone() }
                                    required = true
                                />
                                <div class="field">
                                    <label class="label">{ "Work" }</label>
                                    <div class="control is-expanded">
                                        {&self.new_relation.related_work.full_title}
                                    </div>
                                </div>
                                <FormNumberInput
                                    label = "Relation Ordinal"
                                    value={ self.new_relation.relation_ordinal }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOrdinal(e.to_value())) }
                                    required = true
                                    min={ "1".to_string() }
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="relations-form"
                            >
                                { "Add Related Work" }
                            </button>
                            <button
                                class="button"
                                onclick={ &close_modal }
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !relations.is_empty() {
                        html!{{for relations.iter().map(|r| self.render_relation(ctx, r))}}
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
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_relation(&self, ctx: &Context<Self>, r: &WorkRelationWithRelatedWork) -> Html {
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
                                onclick={ ctx.link().callback(move |_| Msg::ChangeRoute(route.clone())) }
                            >
                                { VIEW_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteWorkRelation(relation_id)) }
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
