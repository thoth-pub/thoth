use thoth_api::model::affiliation::AffiliationWithInstitution;
use thoth_api::model::institution::Institution;
use uuid::Uuid;
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
use crate::models::institution::institutions_query::FetchActionInstitutions;
use crate::models::institution::institutions_query::FetchInstitutions;
use crate::models::institution::institutions_query::InstitutionsRequest;
use crate::models::institution::institutions_query::InstitutionsRequestBody;
use crate::models::institution::institutions_query::Variables as SearchVariables;
use crate::models::Dropdown;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_AFFILIATIONS;
use crate::string::REMOVE_BUTTON;

use super::ToOption;

pub struct AffiliationsFormComponent {
    fetch_affiliations: FetchAffiliations,
    props: Props,
    data: AffiliationsFormData,
    new_affiliation: AffiliationWithInstitution,
    show_add_form: bool,
    show_results: bool,
    fetch_institutions: FetchInstitutions,
    push_affiliation: PushCreateAffiliation,
    delete_affiliation: PushDeleteAffiliation,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct AffiliationsFormData {
    institutions: Vec<Institution>,
    affiliations: Option<Vec<AffiliationWithInstitution>>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetAffiliationsFetchState(FetchActionAffiliations),
    GetAffiliations,
    SetInstitutionsFetchState(FetchActionInstitutions),
    GetInstitutions,
    ToggleSearchResultDisplay(bool),
    SearchInstitution(String),
    SetAffiliationPushState(PushActionCreateAffiliation),
    CreateAffiliation,
    SetAffiliationDeleteState(PushActionDeleteAffiliation),
    DeleteAffiliation(Uuid),
    AddAffiliation(Institution),
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let body = AffiliationsRequestBody {
            variables: Variables {
                contribution_id: props.contribution_id,
            },
            ..Default::default()
        };
        let request = AffiliationsRequest { body };
        let fetch_affiliations = Fetch::new(request);
        let data: AffiliationsFormData = Default::default();
        let new_affiliation: AffiliationWithInstitution = Default::default();
        let show_add_form = false;
        let show_results = false;
        let fetch_institutions = Default::default();
        let push_affiliation = Default::default();
        let delete_affiliation = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        link.send_message(Msg::GetAffiliations);

        AffiliationsFormComponent {
            fetch_affiliations,
            props,
            data,
            new_affiliation,
            show_add_form,
            show_results,
            fetch_institutions,
            push_affiliation,
            delete_affiliation,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
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
                self.link.send_future(
                    self.fetch_affiliations
                        .fetch(Msg::SetAffiliationsFetchState),
                );
                self.link
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
                self.link.send_future(
                    self.fetch_institutions
                        .fetch(Msg::SetInstitutionsFetchState),
                );
                self.link
                    .send_message(Msg::SetInstitutionsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetAffiliationPushState(fetch_state) => {
                self.push_affiliation.apply(fetch_state);
                match self.push_affiliation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_affiliation {
                        Some(i) => {
                            let affiliation = i.clone();
                            let mut affiliations: Vec<AffiliationWithInstitution> =
                                self.data.affiliations.clone().unwrap_or_default();
                            affiliations.push(affiliation);
                            self.data.affiliations = Some(affiliations);
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link.send_message(Msg::ToggleAddFormDisplay(false));
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
                        contribution_id: self.props.contribution_id,
                        institution_id: self.new_affiliation.institution_id,
                        position: self.new_affiliation.position.clone(),
                        affiliation_ordinal: self.new_affiliation.affiliation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateAffiliationRequest { body };
                self.push_affiliation = Fetch::new(request);
                self.link
                    .send_future(self.push_affiliation.fetch(Msg::SetAffiliationPushState));
                self.link
                    .send_message(Msg::SetAffiliationPushState(FetchAction::Fetching));
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
                self.link.send_future(
                    self.delete_affiliation
                        .fetch(Msg::SetAffiliationDeleteState),
                );
                self.link
                    .send_message(Msg::SetAffiliationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddAffiliation(institution) => {
                self.new_affiliation.institution_id = institution.institution_id;
                self.new_affiliation.institution = institution;
                self.link.send_message(Msg::ToggleAddFormDisplay(true));
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
                self.link.send_message(Msg::GetInstitutions);
                false
            }
            Msg::ChangePosition(val) => self
                .new_affiliation
                .position
                .neq_assign(val.to_opt_string()),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.new_affiliation.affiliation_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        log::info!("change");
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        // Ensure the form has a unique ID, as there may be multiple copies of
        // the form on the same parent page, and ID clashes can lead to bugs
        let form_id = format!("affiliations-form-{}", self.props.contribution_id);
        let affiliations = self.data.affiliations.clone().unwrap_or_default();
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Affiliations" }
                </p>
                <div class="panel-block">
                    <div class=self.search_dropdown_status() style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Institution"
                                        aria-haspopup="true"
                                        aria-controls="institutions-menu"
                                        oninput=self.link.callback(|e: InputData| Msg::SearchInstitution(e.value))
                                        onfocus=self.link.callback(|_| Msg::ToggleSearchResultDisplay(true))
                                        onblur=self.link.callback(|_| Msg::ToggleSearchResultDisplay(false))
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
                                            self.link.callback(move |_| {
                                                Msg::AddAffiliation(institution.clone())
                                            })
                                        )
                                    })
                                }
                            </div>
                        </div>
                    </div>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Affiliation" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id={form_id.clone()} onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateAffiliation
                            })
                            >
                                <div class="field">
                                    <label class="label">{ "Institution" }</label>
                                    <div class="control is-expanded">
                                        {&self.new_affiliation.institution.institution_name}
                                    </div>
                                </div>
                                <FormTextInput
                                    label="Position"
                                    value=self.new_affiliation.position.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangePosition(e.value))
                                />
                                <FormNumberInput
                                    label = "Affiliation Ordinal"
                                    value=self.new_affiliation.affiliation_ordinal
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
                                form={form_id.clone()}
                            >
                                { "Add Affiliation" }
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
                    if !affiliations.is_empty() {
                        html!{{for affiliations.iter().map(|a| self.render_affiliation(a))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_AFFILIATIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl AffiliationsFormComponent {
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

    fn render_affiliation(&self, a: &AffiliationWithInstitution) -> Html {
        let affiliation_id = a.affiliation_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-address-card" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Institution" }</label>
                        <div class="control is-expanded">
                            {&a.institution.institution_name}
                        </div>
                    </div>
                </div>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Position" }</label>
                        <div class="control is-expanded">
                            {&a.position.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                </div>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Affiliation Ordinal" }</label>
                        <div class="control is-expanded">
                            {&a.affiliation_ordinal.clone()}
                        </div>
                    </div>
                </div>

                <div class="field">
                    <label class="label"></label>
                    <div class="control is-expanded">
                        <a
                            class="button is-danger"
                            onclick=self.link.callback(move |_| Msg::DeleteAffiliation(affiliation_id))
                        >
                            { REMOVE_BUTTON }
                        </a>
                    </div>
                </div>
            </div>
        }
    }
}
