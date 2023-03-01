use thoth_api::model::funding::FundingWithInstitution;
use thoth_api::model::institution::Institution;
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
use crate::component::institution_select::InstitutionSelectComponent;
use crate::component::utils::FormTextInput;
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
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_FUNDINGS;
use crate::string::REMOVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct FundingsFormComponent {
    new_funding: FundingWithInstitution,
    show_add_form: bool,
    push_funding: PushCreateFunding,
    delete_funding: PushDeleteFunding,
    notification_bus: NotificationDispatcher,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetFundingPushState(PushActionCreateFunding),
    CreateFunding,
    SetFundingDeleteState(PushActionDeleteFunding),
    DeleteFunding(Uuid),
    AddFunding(Institution),
    ChangeProgram(String),
    ChangeProjectName(String),
    ChangeProjectShortname(String),
    ChangeGrant(String),
    ChangeJurisdiction(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub fundings: Option<Vec<FundingWithInstitution>>,
    pub work_id: Uuid,
    pub update_fundings: Callback<Option<Vec<FundingWithInstitution>>>,
}

impl Component for FundingsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        let new_funding: FundingWithInstitution = Default::default();
        let show_add_form = false;
        let push_funding = Default::default();
        let delete_funding = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        FundingsFormComponent {
            new_funding,
            show_add_form,
            push_funding,
            delete_funding,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetFundingPushState(fetch_state) => {
                self.push_funding.apply(fetch_state);
                match self.push_funding.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_funding {
                        Some(i) => {
                            let funding = i.clone();
                            let mut fundings: Vec<FundingWithInstitution> =
                                ctx.props().fundings.clone().unwrap_or_default();
                            fundings.push(funding);
                            ctx.props().update_fundings.emit(Some(fundings));
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
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateFunding => {
                let body = CreateFundingRequestBody {
                    variables: CreateVariables {
                        work_id: ctx.props().work_id,
                        institution_id: self.new_funding.institution_id,
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
                ctx.link()
                    .send_future(self.push_funding.fetch(Msg::SetFundingPushState));
                ctx.link()
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
                            let to_keep: Vec<FundingWithInstitution> = ctx
                                .props()
                                .fundings
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|f| f.funding_id != funding.funding_id)
                                .collect();
                            ctx.props().update_fundings.emit(Some(to_keep));
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
            Msg::DeleteFunding(funding_id) => {
                let body = DeleteFundingRequestBody {
                    variables: DeleteVariables { funding_id },
                    ..Default::default()
                };
                let request = DeleteFundingRequest { body };
                self.delete_funding = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_funding.fetch(Msg::SetFundingDeleteState));
                ctx.link()
                    .send_message(Msg::SetFundingDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddFunding(institution) => {
                self.new_funding.institution_id = institution.institution_id;
                self.new_funding.institution = institution;
                ctx.link().send_message(Msg::ToggleAddFormDisplay(true));
                true
            }
            Msg::ChangeProgram(val) => self.new_funding.program.neq_assign(val.to_opt_string()),
            Msg::ChangeProjectName(val) => self
                .new_funding
                .project_name
                .neq_assign(val.to_opt_string()),
            Msg::ChangeProjectShortname(val) => self
                .new_funding
                .project_shortname
                .neq_assign(val.to_opt_string()),
            Msg::ChangeGrant(val) => self
                .new_funding
                .grant_number
                .neq_assign(val.to_opt_string()),
            Msg::ChangeJurisdiction(val) => self
                .new_funding
                .jurisdiction
                .neq_assign(val.to_opt_string()),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let fundings = ctx.props().fundings.clone().unwrap_or_default();
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        let institution_select_callback = ctx.link().callback(Msg::AddFunding);

        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Funding" }
                </p>
                <div class="panel-block">
                    <InstitutionSelectComponent callback={institution_select_callback} />
                </div>
                <div class={ self.add_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Funding" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="fundings-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateFunding
                            }) }
                            >
                                <div class="field">
                                    <label class="label">{ "Institution" }</label>
                                    <div class="control is-expanded">
                                        {&self.new_funding.institution.institution_name}
                                    </div>
                                </div>
                                <FormTextInput
                                    label="Program"
                                    value={ self.new_funding.program.clone().unwrap_or_default() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeProgram(e.to_value())) }
                                />
                                <FormTextInput
                                    label="Project Name"
                                    value={ self.new_funding.project_name.clone().unwrap_or_default() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeProjectName(e.to_value())) }
                                />
                                <FormTextInput
                                    label="Project Short Name"
                                    value={ self.new_funding.project_shortname.clone().unwrap_or_default() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeProjectShortname(e.to_value())) }
                                />
                                <FormTextInput
                                    label="Grant Number"
                                    value={ self.new_funding.grant_number.clone().unwrap_or_default() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeGrant(e.to_value())) }
                                />
                                <FormTextInput
                                    label="Jurisdiction"
                                    value={ self.new_funding.jurisdiction.clone().unwrap_or_default() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeJurisdiction(e.to_value())) }
                                />

                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="fundings-form"
                            >
                                { "Add Funding" }
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
                    if !fundings.is_empty() {
                        html!{{for fundings.iter().map(|f| self.render_funding(ctx, f))}}
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

    fn render_funding(&self, ctx: &Context<Self>, f: &FundingWithInstitution) -> Html {
        let funding_id = f.funding_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-user" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Institution" }</label>
                        <div class="control is-expanded">
                            {&f.institution.institution_name}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Program" }</label>
                        <div class="control is-expanded">
                            {&f.program.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Project Name" }</label>
                        <div class="control is-expanded">
                            {&f.project_name.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Project Short Name" }</label>
                        <div class="control is-expanded">
                            {&f.project_shortname.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Grant Number" }</label>
                        <div class="control is-expanded">
                            {&f.grant_number.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Jurisdiction" }</label>
                        <div class="control is-expanded">
                            {&f.jurisdiction.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteFunding(funding_id)) }
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
