use thoth_api::funder::model::Funder;
use thoth_api::funding::model::FundingExtended;
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
use crate::component::utils::FormTextInput;
use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::funders_query::FundersRequest;
use crate::models::funder::funders_query::FundersRequestBody;
use crate::models::funder::funders_query::Variables;
use crate::models::funding::create_funding_mutation::CreateFundingRequest;
use crate::models::funding::create_funding_mutation::CreateFundingRequestBody;
use crate::models::funding::create_funding_mutation::PushActionCreateFunding;
use crate::models::funding::create_funding_mutation::PushCreateFunding;
use crate::models::funding::create_funding_mutation::Variables as CreateVariables;
use crate::models::funding::delete_funding_mutation::DeleteFundingRequest;
use crate::models::funding::delete_funding_mutation::DeleteFundingRequestBody;
use crate::models::funding::delete_funding_mutation::PushActionDeleteFunding;
use crate::models::funding::delete_funding_mutation::PushDeleteFunding;
use crate::models::funding::delete_funding_mutation::Variables as DeleteVariables;
use crate::models::Dropdown;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_FUNDINGS;
use crate::string::REMOVE_BUTTON;

pub struct FundingsFormComponent {
    props: Props,
    data: FundingsFormData,
    new_funding: FundingExtended,
    show_add_form: bool,
    show_results: bool,
    fetch_funders: FetchFunders,
    push_funding: PushCreateFunding,
    delete_funding: PushDeleteFunding,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct FundingsFormData {
    funders: Vec<Funder>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetFundersFetchState(FetchActionFunders),
    GetFunders,
    ToggleSearchResultDisplay(bool),
    SearchFunder(String),
    SetFundingPushState(PushActionCreateFunding),
    CreateFunding,
    SetFundingDeleteState(PushActionDeleteFunding),
    DeleteFunding(Uuid),
    AddFunding(Funder),
    ChangeProgram(String),
    ChangeProjectName(String),
    ChangeProjectShortname(String),
    ChangeGrant(String),
    ChangeJurisdiction(String),
    DoNothing,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub fundings: Option<Vec<FundingExtended>>,
    pub work_id: Uuid,
    pub update_fundings: Callback<Option<Vec<FundingExtended>>>,
}

impl Component for FundingsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: FundingsFormData = Default::default();
        let new_funding: FundingExtended = Default::default();
        let show_add_form = false;
        let show_results = false;
        let fetch_funders = Default::default();
        let push_funding = Default::default();
        let delete_funding = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        link.send_message(Msg::GetFunders);

        FundingsFormComponent {
            props,
            data,
            new_funding,
            show_add_form,
            show_results,
            fetch_funders,
            push_funding,
            delete_funding,
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
            Msg::SetFundersFetchState(fetch_state) => {
                self.fetch_funders.apply(fetch_state);
                self.data.funders = match self.fetch_funders.clone().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.funders,
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetFunders => {
                self.link
                    .send_future(self.fetch_funders.fetch(Msg::SetFundersFetchState));
                self.link
                    .send_message(Msg::SetFundersFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetFundingPushState(fetch_state) => {
                self.push_funding.apply(fetch_state);
                match self.push_funding.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_funding {
                        Some(i) => {
                            let funding = i.clone();
                            let mut fundings: Vec<FundingExtended> =
                                self.props.fundings.clone().unwrap_or_default();
                            fundings.push(funding);
                            self.props.update_fundings.emit(Some(fundings));
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
            Msg::CreateFunding => {
                let body = CreateFundingRequestBody {
                    variables: CreateVariables {
                        work_id: self.props.work_id,
                        funder_id: self.new_funding.funder_id,
                        program: self.new_funding.program.clone(),
                        project_name: self.new_funding.project_name.clone(),
                        project_shortname: self.new_funding.project_shortname.clone(),
                        grant_number: self.new_funding.grant_number.clone(),
                        jurisdiction: self.new_funding.jurisdiction.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateFundingRequest { body };
                self.push_funding = Fetch::new(request);
                self.link
                    .send_future(self.push_funding.fetch(Msg::SetFundingPushState));
                self.link
                    .send_message(Msg::SetFundingPushState(FetchAction::Fetching));
                false
            }
            Msg::SetFundingDeleteState(fetch_state) => {
                self.delete_funding.apply(fetch_state);
                match self.delete_funding.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_funding {
                        Some(funding) => {
                            let to_keep: Vec<FundingExtended> = self
                                .props
                                .fundings
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|f| f.funding_id != funding.funding_id)
                                .collect();
                            self.props.update_fundings.emit(Some(to_keep));
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
            Msg::DeleteFunding(funding_id) => {
                let body = DeleteFundingRequestBody {
                    variables: DeleteVariables { funding_id },
                    ..Default::default()
                };
                let request = DeleteFundingRequest { body };
                self.delete_funding = Fetch::new(request);
                self.link
                    .send_future(self.delete_funding.fetch(Msg::SetFundingDeleteState));
                self.link
                    .send_message(Msg::SetFundingDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddFunding(funder) => {
                self.new_funding.funder_id = funder.funder_id;
                self.new_funding.funder = funder;
                self.link.send_message(Msg::ToggleAddFormDisplay(true));
                true
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchFunder(value) => {
                let body = FundersRequestBody {
                    variables: Variables {
                        filter: Some(value),
                        limit: Some(9999),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = FundersRequest { body };
                self.fetch_funders = Fetch::new(request);
                self.link.send_message(Msg::GetFunders);
                false
            }
            Msg::ChangeProgram(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_funding.program.neq_assign(value)
            }
            Msg::ChangeProjectName(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_funding.project_name.neq_assign(value)
            }
            Msg::ChangeProjectShortname(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_funding.project_shortname.neq_assign(value)
            }
            Msg::ChangeGrant(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_funding.grant_number.neq_assign(value)
            }
            Msg::ChangeJurisdiction(val) => {
                let value = match val.is_empty() {
                    true => None,
                    false => Some(val),
                };
                self.new_funding.jurisdiction.neq_assign(value)
            }
            Msg::DoNothing => false, // callbacks need to return a message
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let fundings = self.props.fundings.clone().unwrap_or_default();
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Funding" }
                </p>
                <div class="panel-block">
                    <div class=self.search_dropdown_status() style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Funder"
                                        aria-haspopup="true"
                                        aria-controls="funders-menu"
                                        oninput=self.link.callback(|e: InputData| Msg::SearchFunder(e.value))
                                        onfocus=self.link.callback(|_| Msg::ToggleSearchResultDisplay(true))
                                        onblur=self.link.callback(|_| Msg::ToggleSearchResultDisplay(false))
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                            </div>
                        </div>
                        <div class="dropdown-menu" id="funders-menu" role="menu">
                            <div class="dropdown-content">
                                {
                                    for self.data.funders.iter().map(|f| {
                                        let funder = f.clone();
                                        f.as_dropdown_item(
                                            self.link.callback(move |_| {
                                                Msg::AddFunding(funder.clone())
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
                            <p class="modal-card-title">{ "New Funding" }</p>
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
                                <div class="field">
                                    <label class="label">{ "Funder" }</label>
                                    <div class="control is-expanded">
                                        {&self.new_funding.funder.funder_name}
                                    </div>
                                </div>
                                <FormTextInput
                                    label="Program"
                                    value=self.new_funding.program.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeProgram(e.value))
                                />
                                <FormTextInput
                                    label="Project Name"
                                    value=self.new_funding.project_name.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeProjectName(e.value))
                                />
                                <FormTextInput
                                    label="Project Short Name"
                                    value=self.new_funding.project_shortname.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeProjectShortname(e.value))
                                />
                                <FormTextInput
                                    label="Grant Number"
                                    value=self.new_funding.grant_number.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeGrant(e.value))
                                />
                                <FormTextInput
                                    label="Jurisdiction"
                                    value=self.new_funding.jurisdiction.clone().unwrap_or_else(|| "".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeJurisdiction(e.value))
                                />

                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::CreateFunding
                                })
                            >
                                { "Add Funding" }
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
                    if !fundings.is_empty() {
                        html!{{for fundings.iter().map(|c| self.render_funding(c))}}
                    } else {
                        html! {
                            <div class="notification is-info is-light">
                                { EMPTY_FUNDINGS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl FundingsFormComponent {
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

    fn render_funding(&self, f: &FundingExtended) -> Html {
        let funding_id = f.funding_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-user" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Funder" }</label>
                        <div class="control is-expanded">
                            {&f.funder.funder_name}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Program" }</label>
                        <div class="control is-expanded">
                            {&f.program.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Project Name" }</label>
                        <div class="control is-expanded">
                            {&f.project_name.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Project Short Name" }</label>
                        <div class="control is-expanded">
                            {&f.project_shortname.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Grant Number" }</label>
                        <div class="control is-expanded">
                            {&f.grant_number.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Jurisdiction" }</label>
                        <div class="control is-expanded">
                            {&f.jurisdiction.clone().unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>
                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::DeleteFunding(funding_id))
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
