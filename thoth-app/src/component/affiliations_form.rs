use thoth_api::model::affiliation::AffiliationWithInstitution;
use thoth_api::model::institution::Institution;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormInstitutionSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::models::affiliation::affiliations_query::AffiliationsRequest;
use crate::models::affiliation::affiliations_query::AffiliationsRequestBody;
use crate::models::affiliation::affiliations_query::FetchActionAffiliations;
use crate::models::affiliation::affiliations_query::FetchAffiliations;
use crate::models::affiliation::affiliations_query::Variables;
use crate::models::affiliation::create_affiliation_mutation::CreateAffiliationRequest;
use crate::models::affiliation::create_affiliation_mutation::CreateAffiliationRequestBody;
use crate::models::affiliation::create_affiliation_mutation::PushActionCreateAffiliation;
use crate::models::affiliation::create_affiliation_mutation::PushCreateAffiliation;
use crate::models::affiliation::create_affiliation_mutation::Variables as CreateVariables;
use crate::models::affiliation::delete_affiliation_mutation::DeleteAffiliationRequest;
use crate::models::affiliation::delete_affiliation_mutation::DeleteAffiliationRequestBody;
use crate::models::affiliation::delete_affiliation_mutation::PushActionDeleteAffiliation;
use crate::models::affiliation::delete_affiliation_mutation::PushDeleteAffiliation;
use crate::models::affiliation::delete_affiliation_mutation::Variables as DeleteVariables;
use crate::models::affiliation::update_affiliation_mutation::PushActionUpdateAffiliation;
use crate::models::affiliation::update_affiliation_mutation::PushUpdateAffiliation;
use crate::models::affiliation::update_affiliation_mutation::UpdateAffiliationRequest;
use crate::models::affiliation::update_affiliation_mutation::UpdateAffiliationRequestBody;
use crate::models::affiliation::update_affiliation_mutation::Variables as UpdateVariables;
use crate::models::institution::institutions_query::FetchActionInstitutions;
use crate::models::institution::institutions_query::FetchInstitutions;
use crate::models::institution::institutions_query::InstitutionsRequest;
use crate::models::institution::institutions_query::InstitutionsRequestBody;
use crate::models::institution::institutions_query::Variables as SearchVariables;
use crate::models::Dropdown;
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::REMOVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct AffiliationsFormComponent {
    fetch_affiliations: FetchAffiliations,
    data: AffiliationsFormData,
    affiliation: AffiliationWithInstitution,
    show_modal_form: bool,
    in_edit_mode: bool,
    show_results: bool,
    fetch_institutions: FetchInstitutions,
    create_affiliation: PushCreateAffiliation,
    delete_affiliation: PushDeleteAffiliation,
    update_affiliation: PushUpdateAffiliation,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct AffiliationsFormData {
    institutions: Vec<Institution>,
    affiliations: Option<Vec<AffiliationWithInstitution>>,
}

pub enum Msg {
    ToggleModalFormDisplay(bool, Option<AffiliationWithInstitution>),
    SetAffiliationsFetchState(FetchActionAffiliations),
    GetAffiliations,
    SetInstitutionsFetchState(FetchActionInstitutions),
    GetInstitutions,
    ToggleSearchResultDisplay(bool),
    SearchInstitution(String),
    SetAffiliationCreateState(PushActionCreateAffiliation),
    CreateAffiliation,
    SetAffiliationUpdateState(PushActionUpdateAffiliation),
    UpdateAffiliation,
    SetAffiliationDeleteState(PushActionDeleteAffiliation),
    DeleteAffiliation(Uuid),
    AddAffiliation(Institution),
    ChangeInstitution(Uuid),
    ChangePosition(String),
    ChangeOrdinal(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub contribution_id: Uuid,
}

impl Component for AffiliationsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let fetch_affiliations = Default::default();
        let data: AffiliationsFormData = Default::default();
        let affiliation: AffiliationWithInstitution = Default::default();
        let show_modal_form = false;
        let in_edit_mode = false;
        let show_results = false;
        let fetch_institutions = Default::default();
        let create_affiliation = Default::default();
        let delete_affiliation = Default::default();
        let update_affiliation = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ctx.link().send_message(Msg::GetAffiliations);
        ctx.link().send_message(Msg::GetInstitutions);

        AffiliationsFormComponent {
            fetch_affiliations,
            data,
            affiliation,
            show_modal_form,
            in_edit_mode,
            show_results,
            fetch_institutions,
            create_affiliation,
            delete_affiliation,
            update_affiliation,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, a) => {
                self.show_modal_form = show_form;
                self.in_edit_mode = a.is_some();
                if show_form {
                    let body = InstitutionsRequestBody {
                        variables: SearchVariables {
                            limit: Some(9999),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let request = InstitutionsRequest { body };
                    self.fetch_institutions = Fetch::new(request);
                    ctx.link().send_message(Msg::GetInstitutions);
                    if let Some(affiliation) = a {
                        // Editing existing affiliation: load its current values.
                        self.affiliation = affiliation;
                    }
                }
                true
            }
            Msg::SetAffiliationsFetchState(fetch_state) => {
                self.fetch_affiliations.apply(fetch_state);
                self.data.affiliations = match self.fetch_affiliations.as_ref().state() {
                    FetchState::NotFetching(_) => None,
                    FetchState::Fetching(_) => None,
                    FetchState::Fetched(body) => match &body.data.contribution {
                        Some(c) => c.affiliations.clone(),
                        None => Default::default(),
                    },
                    FetchState::Failed(_, _err) => None,
                };
                true
            }
            Msg::GetAffiliations => {
                let body = AffiliationsRequestBody {
                    variables: Variables {
                        contribution_id: ctx.props().contribution_id,
                    },
                    ..Default::default()
                };
                let request = AffiliationsRequest { body };
                self.fetch_affiliations = Fetch::new(request);

                ctx.link().send_future(
                    self.fetch_affiliations
                        .fetch(Msg::SetAffiliationsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetAffiliationsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetInstitutionsFetchState(fetch_state) => {
                self.fetch_institutions.apply(fetch_state);
                self.data.institutions = match self.fetch_institutions.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.institutions.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetInstitutions => {
                ctx.link().send_future(
                    self.fetch_institutions
                        .fetch(Msg::SetInstitutionsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetInstitutionsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetAffiliationCreateState(fetch_state) => {
                self.create_affiliation.apply(fetch_state);
                match self.create_affiliation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_affiliation {
                        Some(a) => {
                            let affiliation = a.clone();
                            let mut affiliations: Vec<AffiliationWithInstitution> =
                                self.data.affiliations.clone().unwrap_or_default();
                            affiliations.push(affiliation);
                            self.data.affiliations = Some(affiliations);
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link()
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateAffiliation => {
                let body = CreateAffiliationRequestBody {
                    variables: CreateVariables {
                        contribution_id: ctx.props().contribution_id,
                        institution_id: self.affiliation.institution_id,
                        position: self.affiliation.position.clone(),
                        affiliation_ordinal: self.affiliation.affiliation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateAffiliationRequest { body };
                self.create_affiliation = Fetch::new(request);
                ctx.link().send_future(
                    self.create_affiliation
                        .fetch(Msg::SetAffiliationCreateState),
                );
                ctx.link()
                    .send_message(Msg::SetAffiliationCreateState(FetchAction::Fetching));
                false
            }
            Msg::SetAffiliationUpdateState(fetch_state) => {
                self.update_affiliation.apply(fetch_state);
                match self.update_affiliation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_affiliation {
                        Some(a) => {
                            let mut affiliations: Vec<AffiliationWithInstitution> =
                                self.data.affiliations.clone().unwrap_or_default();
                            if let Some(affiliation) = affiliations
                                .iter_mut()
                                .find(|af| af.affiliation_id == a.affiliation_id)
                            {
                                *affiliation = a.clone();
                                self.data.affiliations = Some(affiliations);
                            } else {
                                // This should not be possible: the updated affiliation returned from the
                                // database does not match any of the locally-stored affiliation data.
                                // Refreshing the page will reload the local data from the database.
                                self.notification_bus.send(Request::NotificationBusMsg((
                                    "Changes were saved but display failed to update. Refresh your browser to view current data.".to_string(),
                                    NotificationStatus::Warning,
                                )));
                            }
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link()
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateAffiliation => {
                let body = UpdateAffiliationRequestBody {
                    variables: UpdateVariables {
                        affiliation_id: self.affiliation.affiliation_id,
                        contribution_id: ctx.props().contribution_id,
                        institution_id: self.affiliation.institution_id,
                        position: self.affiliation.position.clone(),
                        affiliation_ordinal: self.affiliation.affiliation_ordinal,
                    },
                    ..Default::default()
                };
                let request = UpdateAffiliationRequest { body };
                self.update_affiliation = Fetch::new(request);
                ctx.link().send_future(
                    self.update_affiliation
                        .fetch(Msg::SetAffiliationUpdateState),
                );
                ctx.link()
                    .send_message(Msg::SetAffiliationUpdateState(FetchAction::Fetching));
                false
            }
            Msg::SetAffiliationDeleteState(fetch_state) => {
                self.delete_affiliation.apply(fetch_state);
                match self.delete_affiliation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_affiliation {
                        Some(affiliation) => {
                            let to_keep: Vec<AffiliationWithInstitution> = self
                                .data
                                .affiliations
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|a| a.affiliation_id != affiliation.affiliation_id)
                                .collect();
                            self.data.affiliations = Some(to_keep);
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
            Msg::DeleteAffiliation(affiliation_id) => {
                let body = DeleteAffiliationRequestBody {
                    variables: DeleteVariables { affiliation_id },
                    ..Default::default()
                };
                let request = DeleteAffiliationRequest { body };
                self.delete_affiliation = Fetch::new(request);
                ctx.link().send_future(
                    self.delete_affiliation
                        .fetch(Msg::SetAffiliationDeleteState),
                );
                ctx.link()
                    .send_message(Msg::SetAffiliationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddAffiliation(institution) => {
                self.affiliation.institution_id = institution.institution_id;
                self.affiliation.institution = institution;
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(true, None));
                true
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchInstitution(value) => {
                let body = InstitutionsRequestBody {
                    variables: SearchVariables {
                        filter: Some(value),
                        limit: Some(9999),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = InstitutionsRequest { body };
                self.fetch_institutions = Fetch::new(request);
                ctx.link().send_message(Msg::GetInstitutions);
                false
            }
            Msg::ChangeInstitution(institution_id) => {
                // ID may be nil if placeholder option was selected.
                // Reset institution anyway, to keep display/underlying values in sync.
                self.affiliation.institution_id.neq_assign(institution_id);
                // we already have the full list of institutions
                if let Some(institution) = self
                    .data
                    .institutions
                    .iter()
                    .find(|i| i.institution_id == institution_id)
                {
                    self.affiliation.institution.neq_assign(institution.clone());
                } else {
                    // Institution not found: clear existing selection
                    self.affiliation.institution.neq_assign(Default::default());
                }
                true
            }
            Msg::ChangePosition(val) => self.affiliation.position.neq_assign(val.to_opt_string()),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.affiliation.affiliation_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        ctx.link().send_message(Msg::GetAffiliations);
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // Ensure the form has a unique ID, as there may be multiple copies of
        // the form on the same parent page, and ID clashes can lead to bugs
        let form_id = format!("affiliations-form-{}", ctx.props().contribution_id);
        let affiliations = self.data.affiliations.clone().unwrap_or_default();
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false, None)
        });
        html! {
            <div class="field">
                <div class={ self.modal_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ self.modal_form_title() }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id={form_id.clone()} onsubmit={ self.modal_form_action(ctx) }>
                                <FormInstitutionSelect
                                    label = "Institution"
                                    value={ self.affiliation.institution_id }
                                    data={ self.data.institutions.clone() }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeInstitution(Uuid::parse_str(&e.to_value()).unwrap_or_default())
                                    ) }
                                    required = true
                                />
                                <FormTextInput
                                    label="Position"
                                    value={ self.affiliation.position.clone().unwrap_or_else(|| "".to_string()) }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePosition(e.to_value())) }
                                />
                                <FormNumberInput
                                    label = "Affiliation Ordinal"
                                    value={ self.affiliation.affiliation_ordinal }
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
                                form={form_id.clone()}
                            >
                                { self.modal_form_button() }
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
                <table class="table is-fullwidth is-narrow">
                    <thead>
                        <tr>
                            <th class="th">
                                { "Institution" }
                            </th>
                            <th class="th">
                                { "Position" }
                            </th>
                            <th class="th">
                                { "Affiliation Ordinal" }
                            </th>
                            // Empty columns for "Edit" and "Remove" buttons
                            <th class="th"></th>
                            <th class="th"></th>
                        </tr>
                    </thead>
                    <tbody>
                        {for affiliations.iter().map(|a| self.render_affiliation(ctx, a))}
                        <tr class="row">
                            <div class={ self.search_dropdown_status() } style="width: 100%">
                                <div class="dropdown-trigger" style="width: 100%">
                                    <div class="field">
                                        <p class="control is-expanded has-icons-left">
                                            <input
                                                class="input"
                                                type="search"
                                                placeholder="Search Institution"
                                                aria-haspopup="true"
                                                aria-controls="institutions-menu"
                                                oninput={ ctx.link().callback(|e: InputEvent| Msg::SearchInstitution(e.to_value())) }
                                                onfocus={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(true)) }
                                                onblur={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(false)) }
                                            />
                                            <span class="icon is-left">
                                                <i class="fas fa-search" aria-hidden="true"></i>
                                            </span>
                                        </p>
                                    </div>
                                </div>
                                <div class="dropdown-menu" id="institutions-menu" role="menu">
                                    <div class="dropdown-content">
                                        {
                                            for self.data.institutions.iter().map(|i| {
                                                let institution = i.clone();
                                                i.as_dropdown_item(
                                                    ctx.link().callback(move |_| {
                                                        Msg::AddAffiliation(institution.clone())
                                                    })
                                                )
                                            })
                                        }
                                    </div>
                                </div>
                            </div>
                        </tr>
                    </tbody>
                </table>
            </div>
        }
    }
}

impl AffiliationsFormComponent {
    fn modal_form_status(&self) -> String {
        match self.show_modal_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn modal_form_title(&self) -> String {
        match self.in_edit_mode {
            true => "Edit Affiliation".to_string(),
            false => "New Affiliation".to_string(),
        }
    }

    fn modal_form_button(&self) -> String {
        match self.in_edit_mode {
            true => "Save Affiliation".to_string(),
            false => "Add Affiliation".to_string(),
        }
    }

    fn modal_form_action(&self, ctx: &Context<Self>) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateAffiliation
            }),
            false => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateAffiliation
            }),
        }
    }

    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_affiliation(&self, ctx: &Context<Self>, a: &AffiliationWithInstitution) -> Html {
        let affiliation = a.clone();
        let affiliation_id = a.affiliation_id;
        html! {
            <tr class="row">
                <td>{&a.institution.institution_name}</td>
                <td>{&a.position.clone().unwrap_or_else(|| "".to_string())}</td>
                <td>{&a.affiliation_ordinal.clone()}</td>
                <td>
                    <a
                        class="button is-success is-small"
                        onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true, Some(affiliation.clone()))) }
                    >
                        { EDIT_BUTTON }
                    </a>
                </td>
                <td>
                    <a
                        class="button is-danger is-small"
                        onclick={ ctx.link().callback(move |_| Msg::DeleteAffiliation(affiliation_id)) }
                    >
                        { REMOVE_BUTTON }
                    </a>
                </td>
            </tr>
        }
    }
}
