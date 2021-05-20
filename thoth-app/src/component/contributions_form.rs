use std::str::FromStr;
use thoth_api::contribution::model::Contribution;
use thoth_api::contribution::model::ContributionType;
use thoth_api::contributor::model::Contributor;
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
use crate::component::utils::FormBooleanSelect;
use crate::component::utils::FormContributionTypeSelect;
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
use crate::models::contribution::ContributionTypeValues;
use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use crate::models::Dropdown;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_CONTRIBUTIONS;
use crate::string::NO;
use crate::string::REMOVE_BUTTON;
use crate::string::YES;

pub struct ContributionsFormComponent {
    props: Props,
    data: ContributionsFormData,
    new_contribution: Contribution,
    show_add_form: bool,
    show_results: bool,
    fetch_contributors: FetchContributors,
    fetch_contribution_types: FetchContributionTypes,
    push_contribution: PushCreateContribution,
    delete_contribution: PushDeleteContribution,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct ContributionsFormData {
    contributors: Vec<Contributor>,
    contribution_types: Vec<ContributionTypeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetContributorsFetchState(FetchActionContributors),
    GetContributors,
    SetContributionTypesFetchState(FetchActionContributionTypes),
    GetContributionTypes,
    ToggleSearchResultDisplay(bool),
    SearchContributor(String),
    SetContributionPushState(PushActionCreateContribution),
    CreateContribution,
    SetContributionDeleteState(PushActionDeleteContribution),
    DeleteContribution(Uuid),
    AddContribution(Contributor),
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeInstitution(String),
    ChangeBiography(String),
    ChangeContributiontype(ContributionType),
    ChangeMainContribution(bool),
    DoNothing,
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: ContributionsFormData = Default::default();
        let new_contribution: Contribution = Default::default();
        let show_add_form = false;
        let show_results = false;
        let fetch_contributors = Default::default();
        let fetch_contribution_types = Default::default();
        let push_contribution = Default::default();
        let delete_contribution = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        link.send_message(Msg::GetContributors);
        link.send_message(Msg::GetContributionTypes);

        ContributionsFormComponent {
            props,
            data,
            new_contribution,
            show_add_form,
            show_results,
            fetch_contributors,
            fetch_contribution_types,
            push_contribution,
            delete_contribution,
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
                self.link.send_future(
                    self.fetch_contributors
                        .fetch(Msg::SetContributorsFetchState),
                );
                self.link
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
                self.link.send_future(
                    self.fetch_contribution_types
                        .fetch(Msg::SetContributionTypesFetchState),
                );
                self.link
                    .send_message(Msg::SetContributionTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionPushState(fetch_state) => {
                self.push_contribution.apply(fetch_state);
                match self.push_contribution.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_contribution {
                        Some(i) => {
                            let contribution = i.clone();
                            let mut contributions: Vec<Contribution> =
                                self.props.contributions.clone().unwrap_or_default();
                            contributions.push(contribution);
                            self.props.update_contributions.emit(Some(contributions));
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
            Msg::CreateContribution => {
                let body = CreateContributionRequestBody {
                    variables: CreateVariables {
                        work_id: self.props.work_id,
                        contributor_id: self.new_contribution.contributor_id,
                        contribution_type: self.new_contribution.contribution_type,
                        main_contribution: self.new_contribution.main_contribution,
                        biography: self.new_contribution.biography.clone(),
                        institution: self.new_contribution.institution.clone(),
                        first_name: self.new_contribution.first_name.clone(),
                        last_name: self.new_contribution.last_name.clone(),
                        full_name: self.new_contribution.full_name.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateContributionRequest { body };
                self.push_contribution = Fetch::new(request);
                self.link
                    .send_future(self.push_contribution.fetch(Msg::SetContributionPushState));
                self.link
                    .send_message(Msg::SetContributionPushState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionDeleteState(fetch_state) => {
                self.delete_contribution.apply(fetch_state);
                match self.delete_contribution.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_contribution {
                        Some(contribution) => {
                            let to_keep: Vec<Contribution> = self
                                .props
                                .contributions
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|c| {
                                    c.contributor_id != contribution.contributor_id
                                        || c.contribution_type != contribution.contribution_type
                                })
                                .collect();
                            self.props.update_contributions.emit(Some(to_keep));
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
            Msg::DeleteContribution(contribution_id) => {
                let body = DeleteContributionRequestBody {
                    variables: DeleteVariables { contribution_id },
                    ..Default::default()
                };
                let request = DeleteContributionRequest { body };
                self.delete_contribution = Fetch::new(request);
                self.link.send_future(
                    self.delete_contribution
                        .fetch(Msg::SetContributionDeleteState),
                );
                self.link
                    .send_message(Msg::SetContributionDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddContribution(contributor) => {
                self.new_contribution.contributor_id = contributor.contributor_id;
                self.new_contribution.first_name = contributor.first_name;
                self.new_contribution.last_name = contributor.last_name;
                self.new_contribution.full_name = contributor.full_name;
                self.link.send_message(Msg::ToggleAddFormDisplay(true));
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
                self.link.send_message(Msg::GetContributors);
                false
            }
            Msg::ChangeFirstName(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_contribution.first_name.neq_assign(value)
            }
            Msg::ChangeLastName(val) => self.new_contribution.last_name.neq_assign(val),
            Msg::ChangeFullName(val) => self.new_contribution.full_name.neq_assign(val),
            Msg::ChangeInstitution(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_contribution.institution.neq_assign(value)
            }
            Msg::ChangeBiography(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_contribution.biography.neq_assign(value)
            }
            Msg::ChangeContributiontype(val) => {
                self.new_contribution.contribution_type.neq_assign(val)
            }
            Msg::ChangeMainContribution(val) => {
                self.new_contribution.main_contribution.neq_assign(val)
            }
            Msg::DoNothing => false, // callbacks need to return a message
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let contributions = self.props.contributions.clone().unwrap_or_default();
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Contributions" }
                </p>
                <div class="panel-block">
                    <div class=self.search_dropdown_status() style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Contributor"
                                        aria-haspopup="true"
                                        aria-controls="contributors-menu"
                                        oninput=self.link.callback(|e: InputData| Msg::SearchContributor(e.value))
                                        onfocus=self.link.callback(|_| Msg::ToggleSearchResultDisplay(true))
                                        onblur=self.link.callback(|_| Msg::ToggleSearchResultDisplay(false))
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
                                            self.link.callback(move |_| {
                                                Msg::AddContribution(contributor.clone())
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
                            <p class="modal-card-title">{ "New Contribution" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::DoNothing
                            })
                            >
                                <FormTextInput
                                    label="Contributor's Given Name"
                                    value=self.new_contribution.first_name.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeFirstName(e.value))
                                />
                                <FormTextInput
                                    label="Contributor's Family Name"
                                    value=self.new_contribution.last_name.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeLastName(e.value))
                                />
                                <FormTextInput
                                    label="Contributor's Full Name"
                                    value=self.new_contribution.full_name.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeFullName(e.value))
                                />
                                <FormContributionTypeSelect
                                    label = "Contribution Type"
                                    value=self.new_contribution.contribution_type
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeContributiontype(ContributionType::from_str(&value).unwrap())
                                        }
                                        _ => unreachable!(),
                                    })
                                    data=self.data.contribution_types.clone()
                                    required = true
                                />
                                <FormTextInput
                                    label="Institution"
                                    value=self.new_contribution.institution.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeInstitution(e.value))
                                />
                                <FormTextInput
                                    label="Biography"
                                    value=self.new_contribution.biography.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeBiography(e.value))
                                />
                                <FormBooleanSelect
                                    label = "Main"
                                    value=self.new_contribution.main_contribution
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            let boolean = value == "true";
                                            Msg::ChangeMainContribution(boolean)
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::CreateContribution
                                })
                            >
                                { "Add Contribution" }
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
                    if !contributions.is_empty() {
                        html!{{for contributions.iter().map(|c| self.render_contribution(c))}}
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

    fn render_contribution(&self, c: &Contribution) -> Html {
        let contribution_id = c.contribution_id;
        html! {
            <div class="panel-block field is-horizontal">
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
                        <label class="label">{ "Institution" }</label>
                        <div class="control is-expanded">
                            {&c.institution.clone().unwrap_or_else(|| "".to_string())}
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

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::DeleteContribution(contribution_id))
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
