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
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::REMOVE_BUTTON;

use super::ToOption;

pub struct AffiliationsFormComponent {
    fetch_affiliations: FetchAffiliations,
    props: Props,
    data: AffiliationsFormData,
    affiliation: AffiliationWithInstitution,
    show_modal_form: bool,
    in_edit_mode: bool,
    fetch_institutions: FetchInstitutions,
    create_affiliation: PushCreateAffiliation,
    delete_affiliation: PushDeleteAffiliation,
    update_affiliation: PushUpdateAffiliation,
    link: ComponentLink<Self>,
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
    SetAffiliationCreateState(PushActionCreateAffiliation),
    CreateAffiliation,
    SetAffiliationUpdateState(PushActionUpdateAffiliation),
    UpdateAffiliation,
    SetAffiliationDeleteState(PushActionDeleteAffiliation),
    DeleteAffiliation(Uuid),
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let fetch_affiliations = Default::default();
        let data: AffiliationsFormData = Default::default();
        let affiliation: AffiliationWithInstitution = Default::default();
        let show_modal_form = false;
        let in_edit_mode = false;
        let body = InstitutionsRequestBody {
            variables: SearchVariables {
                limit: Some(9999),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = InstitutionsRequest { body };
        let fetch_institutions = Fetch::new(request);
        let create_affiliation = Default::default();
        let delete_affiliation = Default::default();
        let update_affiliation = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        link.send_message(Msg::GetAffiliations);
        link.send_message(Msg::GetInstitutions);

        AffiliationsFormComponent {
            fetch_affiliations,
            props,
            data,
            affiliation,
            show_modal_form,
            in_edit_mode,
            fetch_institutions,
            create_affiliation,
            delete_affiliation,
            update_affiliation,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, a) => {
                self.show_modal_form = show_form;
                self.in_edit_mode = a.is_some();
                if show_form {
                    if let Some(affiliation) = a {
                        // Editing existing affiliation: load its current values.
                        self.affiliation = affiliation;
                    } else {
                        self.affiliation.institution_id = Default::default();
                        self.affiliation.institution = Default::default();
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
                        contribution_id: self.props.contribution_id,
                    },
                    ..Default::default()
                };
                let request = AffiliationsRequest { body };
                self.fetch_affiliations = Fetch::new(request);

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
            Msg::CreateAffiliation => {
                let body = CreateAffiliationRequestBody {
                    variables: CreateVariables {
                        contribution_id: self.props.contribution_id,
                        institution_id: self.affiliation.institution_id,
                        position: self.affiliation.position.clone(),
                        affiliation_ordinal: self.affiliation.affiliation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateAffiliationRequest { body };
                self.create_affiliation = Fetch::new(request);
                self.link.send_future(
                    self.create_affiliation
                        .fetch(Msg::SetAffiliationCreateState),
                );
                self.link
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
            Msg::UpdateAffiliation => {
                let body = UpdateAffiliationRequestBody {
                    variables: UpdateVariables {
                        affiliation_id: self.affiliation.affiliation_id,
                        contribution_id: self.props.contribution_id,
                        institution_id: self.affiliation.institution_id,
                        position: self.affiliation.position.clone(),
                        affiliation_ordinal: self.affiliation.affiliation_ordinal,
                    },
                    ..Default::default()
                };
                let request = UpdateAffiliationRequest { body };
                self.update_affiliation = Fetch::new(request);
                self.link.send_future(
                    self.update_affiliation
                        .fetch(Msg::SetAffiliationUpdateState),
                );
                self.link
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
                self.link.send_future(
                    self.delete_affiliation
                        .fetch(Msg::SetAffiliationDeleteState),
                );
                self.link
                    .send_message(Msg::SetAffiliationDeleteState(FetchAction::Fetching));
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

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.neq_assign(props) {
            self.link.send_message(Msg::GetAffiliations);
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        // Ensure the form has a unique ID, as there may be multiple copies of
        // the form on the same parent page, and ID clashes can lead to bugs
        let form_id = format!("affiliations-form-{}", self.props.contribution_id);
        let affiliations = self.data.affiliations.clone().unwrap_or_default();
        let open_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(true, None)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false, None)
        });
        html! {
            <div class="field">
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
                            <form id={form_id.clone()} onsubmit=self.modal_form_action()>
                                <FormInstitutionSelect
                                    label = "Institution"
                                    value=self.affiliation.institution_id
                                    data=self.data.institutions.clone()
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeInstitution(Uuid::parse_str(&value).unwrap_or_default())
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormTextInput
                                    label="Position"
                                    value=self.affiliation.position.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangePosition(e.value))
                                />
                                <FormNumberInput
                                    label = "Affiliation Ordinal"
                                    value=self.affiliation.affiliation_ordinal
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
                        {for affiliations.iter().map(|a| self.render_affiliation(a))}
                        <tr class="row">
                            <div class="panel-block">
                                <button
                                    class="button is-link is-outlined is-success"
                                    onclick=open_modal
                                >
                                    { "Add Affiliation" }
                                </button>
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

    fn modal_form_action(&self) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => self.link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateAffiliation
            }),
            false => self.link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateAffiliation
            }),
        }
    }

    fn render_affiliation(&self, a: &AffiliationWithInstitution) -> Html {
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
                        onclick=self.link.callback(move |_| Msg::ToggleModalFormDisplay(true, Some(affiliation.clone())))
                    >
                        { EDIT_BUTTON }
                    </a>
                </td>
                <td>
                    <a
                        class="button is-danger is-small"
                        onclick=self.link.callback(move |_| Msg::DeleteAffiliation(affiliation_id))
                    >
                        { REMOVE_BUTTON }
                    </a>
                </td>
            </tr>
        }
    }
}
