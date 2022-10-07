use std::str::FromStr;
use thoth_api::model::contribution::Contribution;
use thoth_api::model::contribution::ContributionType;
use thoth_api::model::contributor::Contributor;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::affiliations_form::AffiliationsFormComponent;
use crate::component::utils::FormBooleanSelect;
use crate::component::utils::FormContributionTypeSelect;
use crate::component::utils::FormContributorSelect;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::models::contribution::contribution_types_query::FetchActionContributionTypes;
use crate::models::contribution::contribution_types_query::FetchContributionTypes;
use crate::models::contribution::create_contribution_mutation::CreateContributionRequest;
use crate::models::contribution::create_contribution_mutation::CreateContributionRequestBody;
use crate::models::contribution::create_contribution_mutation::PushActionCreateContribution;
use crate::models::contribution::create_contribution_mutation::PushCreateContribution;
use crate::models::contribution::create_contribution_mutation::Variables as CreateVariables;
use crate::models::contribution::delete_contribution_mutation::DeleteContributionRequest;
use crate::models::contribution::delete_contribution_mutation::DeleteContributionRequestBody;
use crate::models::contribution::delete_contribution_mutation::PushActionDeleteContribution;
use crate::models::contribution::delete_contribution_mutation::PushDeleteContribution;
use crate::models::contribution::delete_contribution_mutation::Variables as DeleteVariables;
use crate::models::contribution::update_contribution_mutation::PushActionUpdateContribution;
use crate::models::contribution::update_contribution_mutation::PushUpdateContribution;
use crate::models::contribution::update_contribution_mutation::UpdateContributionRequest;
use crate::models::contribution::update_contribution_mutation::UpdateContributionRequestBody;
use crate::models::contribution::update_contribution_mutation::Variables as UpdateVariables;
use crate::models::contribution::ContributionTypeValues;
use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use crate::models::Dropdown;
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_CONTRIBUTIONS;
use crate::string::NO;
use crate::string::REMOVE_BUTTON;
use crate::string::YES;

use super::ToElementValue;
use super::ToOption;

pub struct ContributionsFormComponent {
    data: ContributionsFormData,
    contribution: Contribution,
    show_modal_form: bool,
    in_edit_mode: bool,
    show_results: bool,
    fetch_contributors: FetchContributors,
    fetch_contribution_types: FetchContributionTypes,
    create_contribution: PushCreateContribution,
    delete_contribution: PushDeleteContribution,
    update_contribution: PushUpdateContribution,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct ContributionsFormData {
    contributors: Vec<Contributor>,
    contribution_types: Vec<ContributionTypeValues>,
}

pub enum Msg {
    ToggleModalFormDisplay(bool, Option<Contribution>),
    SetContributorsFetchState(FetchActionContributors),
    GetContributors,
    SetContributionTypesFetchState(FetchActionContributionTypes),
    GetContributionTypes,
    ToggleSearchResultDisplay(bool),
    SearchContributor(String),
    SetContributionCreateState(PushActionCreateContribution),
    CreateContribution,
    SetContributionUpdateState(PushActionUpdateContribution),
    UpdateContribution,
    SetContributionDeleteState(PushActionDeleteContribution),
    DeleteContribution(Uuid),
    AddContribution(Contributor),
    ChangeContributor(Uuid),
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeBiography(String),
    ChangeContributiontype(ContributionType),
    ChangeMainContribution(bool),
    ChangeOrdinal(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub contributions: Option<Vec<Contribution>>,
    pub work_id: Uuid,
    pub update_contributions: Callback<Option<Vec<Contribution>>>,
}

impl Component for ContributionsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let data: ContributionsFormData = Default::default();
        let contribution: Contribution = Default::default();
        let show_modal_form = false;
        let in_edit_mode = false;
        let show_results = false;
        let fetch_contributors = Default::default();
        let fetch_contribution_types = Default::default();
        let create_contribution = Default::default();
        let delete_contribution = Default::default();
        let update_contribution = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ctx.link().send_message(Msg::GetContributors);
        ctx.link().send_message(Msg::GetContributionTypes);

        ContributionsFormComponent {
            data,
            contribution,
            show_modal_form,
            in_edit_mode,
            show_results,
            fetch_contributors,
            fetch_contribution_types,
            create_contribution,
            delete_contribution,
            update_contribution,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, c) => {
                self.show_modal_form = show_form;
                self.in_edit_mode = c.is_some();
                if show_form {
                    let body = ContributorsRequestBody {
                        variables: Variables {
                            limit: Some(9999),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let request = ContributorsRequest { body };
                    self.fetch_contributors = Fetch::new(request);
                    ctx.link().send_message(Msg::GetContributors);
                    if let Some(contribution) = c {
                        // Editing existing contribution: load its current values.
                        self.contribution = contribution;
                    }
                }
                true
            }
            Msg::SetContributorsFetchState(fetch_state) => {
                self.fetch_contributors.apply(fetch_state);
                self.data.contributors = match self.fetch_contributors.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.contributors.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetContributors => {
                ctx.link().send_future(
                    self.fetch_contributors
                        .fetch(Msg::SetContributorsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetContributorsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionTypesFetchState(fetch_state) => {
                self.fetch_contribution_types.apply(fetch_state);
                self.data.contribution_types = match self.fetch_contribution_types.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.contribution_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetContributionTypes => {
                ctx.link().send_future(
                    self.fetch_contribution_types
                        .fetch(Msg::SetContributionTypesFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionCreateState(fetch_state) => {
                self.create_contribution.apply(fetch_state);
                match self.create_contribution.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_contribution {
                        Some(i) => {
                            let contribution = i.clone();
                            let mut contributions: Vec<Contribution> =
                                ctx.props().contributions.clone().unwrap_or_default();
                            contributions.push(contribution);
                            ctx.props().update_contributions.emit(Some(contributions));
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
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateContribution => {
                let body = CreateContributionRequestBody {
                    variables: CreateVariables {
                        work_id: ctx.props().work_id,
                        contributor_id: self.contribution.contributor_id,
                        contribution_type: self.contribution.contribution_type,
                        main_contribution: self.contribution.main_contribution,
                        biography: self.contribution.biography.clone(),
                        first_name: self.contribution.first_name.clone(),
                        last_name: self.contribution.last_name.clone(),
                        full_name: self.contribution.full_name.clone(),
                        contribution_ordinal: self.contribution.contribution_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateContributionRequest { body };
                self.create_contribution = Fetch::new(request);
                ctx.link().send_future(
                    self.create_contribution
                        .fetch(Msg::SetContributionCreateState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionCreateState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionUpdateState(fetch_state) => {
                self.update_contribution.apply(fetch_state);
                match self.update_contribution.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_contribution {
                        Some(c) => {
                            let mut contributions: Vec<Contribution> =
                                ctx.props().contributions.clone().unwrap_or_default();
                            if let Some(contribution) = contributions
                                .iter_mut()
                                .find(|cn| cn.contribution_id == c.contribution_id)
                            {
                                *contribution = c.clone();
                                ctx.props().update_contributions.emit(Some(contributions));
                            } else {
                                // This should not be possible: the updated contribution returned from the
                                // database does not match any of the locally-stored contribution data.
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
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateContribution => {
                let body = UpdateContributionRequestBody {
                    variables: UpdateVariables {
                        contribution_id: self.contribution.contribution_id,
                        work_id: ctx.props().work_id,
                        contributor_id: self.contribution.contributor_id,
                        contribution_type: self.contribution.contribution_type,
                        main_contribution: self.contribution.main_contribution,
                        biography: self.contribution.biography.clone(),
                        first_name: self.contribution.first_name.clone(),
                        last_name: self.contribution.last_name.clone(),
                        full_name: self.contribution.full_name.clone(),
                        contribution_ordinal: self.contribution.contribution_ordinal,
                    },
                    ..Default::default()
                };
                let request = UpdateContributionRequest { body };
                self.update_contribution = Fetch::new(request);
                ctx.link().send_future(
                    self.update_contribution
                        .fetch(Msg::SetContributionUpdateState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionUpdateState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionDeleteState(fetch_state) => {
                self.delete_contribution.apply(fetch_state);
                match self.delete_contribution.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_contribution {
                        Some(contribution) => {
                            let to_keep: Vec<Contribution> = ctx
                                .props()
                                .contributions
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|c| c.contribution_id != contribution.contribution_id)
                                .collect();
                            ctx.props().update_contributions.emit(Some(to_keep));
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
            Msg::DeleteContribution(contribution_id) => {
                let body = DeleteContributionRequestBody {
                    variables: DeleteVariables { contribution_id },
                    ..Default::default()
                };
                let request = DeleteContributionRequest { body };
                self.delete_contribution = Fetch::new(request);
                ctx.link().send_future(
                    self.delete_contribution
                        .fetch(Msg::SetContributionDeleteState),
                );
                ctx.link()
                    .send_message(Msg::SetContributionDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddContribution(contributor) => {
                self.contribution.contributor_id = contributor.contributor_id;
                self.contribution.first_name = contributor.first_name;
                self.contribution.last_name = contributor.last_name;
                self.contribution.full_name = contributor.full_name;
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(true, None));
                true
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchContributor(value) => {
                let body = ContributorsRequestBody {
                    variables: Variables {
                        filter: Some(value),
                        limit: Some(9999),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = ContributorsRequest { body };
                self.fetch_contributors = Fetch::new(request);
                ctx.link().send_message(Msg::GetContributors);
                false
            }
            Msg::ChangeContributor(contributor_id) => {
                // ID may be nil if placeholder option was selected.
                // Reset contributor anyway, to keep display/underlying values in sync.
                self.contribution.contributor_id.neq_assign(contributor_id);
                // we already have the full list of contributors
                if let Some(contributor) = self
                    .data
                    .contributors
                    .iter()
                    .find(|c| c.contributor_id == contributor_id)
                {
                    // Update user-editable name fields to default to canonical name
                    self.contribution
                        .first_name
                        .neq_assign(contributor.first_name.clone());
                    self.contribution
                        .last_name
                        .neq_assign(contributor.last_name.clone());
                    self.contribution
                        .full_name
                        .neq_assign(contributor.full_name.clone());
                }
                true
            }
            Msg::ChangeFirstName(val) => {
                self.contribution.first_name.neq_assign(val.to_opt_string())
            }
            Msg::ChangeLastName(val) => self
                .contribution
                .last_name
                .neq_assign(val.trim().to_owned()),
            Msg::ChangeFullName(val) => self
                .contribution
                .full_name
                .neq_assign(val.trim().to_owned()),
            Msg::ChangeBiography(val) => {
                self.contribution.biography.neq_assign(val.to_opt_string())
            }
            Msg::ChangeContributiontype(val) => self.contribution.contribution_type.neq_assign(val),
            Msg::ChangeMainContribution(val) => self.contribution.main_contribution.neq_assign(val),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.contribution.contribution_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let contributions = ctx.props().contributions.clone().unwrap_or_default();
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false, None)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Contributions" }
                </p>
                <div class="panel-block">
                    <div class={ self.search_dropdown_status() } style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Contributor"
                                        aria-haspopup="true"
                                        aria-controls="contributors-menu"
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::SearchContributor(e.to_value())) }
                                        onfocus={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(true)) }
                                        onblur={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(false)) }
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                            </div>
                        </div>
                        <div class="dropdown-menu" id="contributors-menu" role="menu">
                            <div class="dropdown-content">
                                {
                                    for self.data.contributors.iter().map(|c| {
                                        let contributor = c.clone();
                                        c.as_dropdown_item(
                                            ctx.link().callback(move |_| {
                                                Msg::AddContribution(contributor.clone())
                                            })
                                        )
                                    })
                                }
                            </div>
                        </div>
                    </div>
                </div>
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
                            <form id="contributions-form" onsubmit={ self.modal_form_action(ctx) }>
                                <FormContributorSelect
                                    label = "Contributor"
                                    value={ self.contribution.contributor_id }
                                    data={ self.data.contributors.clone() }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeContributor(Uuid::parse_str(&e.to_value()).unwrap_or_default())
                                    ) }
                                    required = true
                                />
                                <FormTextInput
                                    label="Contributor's Given Name"
                                    value={ self.contribution.first_name.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFirstName(e.to_value())) }
                                />
                                <FormTextInput
                                    label="Contributor's Family Name"
                                    value={ self.contribution.last_name.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLastName(e.to_value())) }
                                    required = true
                                />
                                <FormTextInput
                                    label="Contributor's Full Name"
                                    value={ self.contribution.full_name.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFullName(e.to_value())) }
                                    required = true
                                />
                                <FormContributionTypeSelect
                                    label = "Contribution Type"
                                    value={ self.contribution.contribution_type }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeContributiontype(ContributionType::from_str(&e.to_value()).unwrap())
                                    ) }
                                    data={ self.data.contribution_types.clone() }
                                    required = true
                                />
                                <FormTextInput
                                    label="Biography"
                                    value={ self.contribution.biography.clone().unwrap_or_else(|| "".to_string()) }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeBiography(e.to_value())) }
                                />
                                <FormBooleanSelect
                                    label = "Main"
                                    value={ self.contribution.main_contribution }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeMainContribution(e.to_value() == "true")
                                    ) }
                                    required = true
                                />
                                <FormNumberInput
                                    label = "Contribution Ordinal"
                                    value={ self.contribution.contribution_ordinal }
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
                                form="contributions-form"
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
                {
                    if !contributions.is_empty() {
                        html!{{for contributions.iter().map(|c| self.render_contribution(ctx, c))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_CONTRIBUTIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl ContributionsFormComponent {
    fn modal_form_status(&self) -> String {
        match self.show_modal_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn modal_form_title(&self) -> String {
        match self.in_edit_mode {
            true => "Edit Contribution".to_string(),
            false => "New Contribution".to_string(),
        }
    }

    fn modal_form_button(&self) -> String {
        match self.in_edit_mode {
            true => "Save Contribution".to_string(),
            false => "Add Contribution".to_string(),
        }
    }

    fn modal_form_action(&self, ctx: &Context<Self>) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateContribution
            }),
            false => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateContribution
            }),
        }
    }

    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_contribution(&self, ctx: &Context<Self>, c: &Contribution) -> Html {
        let contribution = c.clone();
        let contribution_id = c.contribution_id;
        html! {
            <div class="panel-block field is-horizontal is-flex-wrap-wrap">
                <span class="panel-icon">
                    <i class="fas fa-user" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Full Name" }</label>
                        <div class="control is-expanded">
                            {&c.full_name}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Contribution Type" }</label>
                        <div class="control is-expanded">
                            {&c.contribution_type}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Biography" }</label>
                        <div class="control is-expanded">
                            {&c.biography.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Main" }</label>
                        <div class="control is-expanded">
                            {
                                match c.main_contribution {
                                    true => { YES },
                                    false => { NO }
                                }
                            }
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Contribution Ordinal" }</label>
                        <div class="control is-expanded">
                            {&c.contribution_ordinal.clone()}
                        </div>
                    </div>

                    <div class="field is-grouped is-grouped-right">
                        <div class="control">
                            <a
                                class="button is-success"
                                onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true, Some(contribution.clone()))) }
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteContribution(contribution_id)) }
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
                <AffiliationsFormComponent
                    contribution_id={ c.contribution_id }
                />
            </div>
        }
    }
}
