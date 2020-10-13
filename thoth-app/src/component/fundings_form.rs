use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::component::utils::FormTextInput;
use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::funders_query::FundersRequest;
use crate::models::funder::funders_query::FundersRequestBody;
use crate::models::funder::funders_query::Variables;
use crate::models::funder::funders_query::FUNDERS_QUERY;
use crate::models::funder::Funder;
use crate::models::funding::Funding;
use crate::string::EMPTY_FUNDINGS;
use crate::string::REMOVE_BUTTON;

pub struct FundingsFormComponent {
    props: Props,
    data: FundingsFormData,
    program_value: String,
    project_name_value: String,
    project_shortname_value: String,
    grant_number_value: String,
    jurisdiction_value: String,
    show_results: bool,
    fetch_funders: FetchFunders,
    link: ComponentLink<Self>,
}

struct FundingsFormData {
    funders: Vec<Funder>,
}

pub enum Msg {
    SetFundersFetchState(FetchActionFunders),
    GetFunders,
    ToggleSearchResultDisplay(bool),
    SearchFunder(String),
    AddFunding(Funder),
    RemoveFunding(String),
    ChangeProgramEditValue(String),
    ChangeProgram(String),
    ChangeProjectNameEditValue(String),
    ChangeProjectName(String),
    ChangeProjectShortnameEditValue(String),
    ChangeProjectShortname(String),
    ChangeGrantEditValue(String),
    ChangeGrant(String),
    ChangeJurisdictionEditValue(String),
    ChangeJurisdiction(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub fundings: Option<Vec<Funding>>,
    pub work_id: String,
    pub update_fundings: Callback<Option<Vec<Funding>>>,
}

impl Component for FundingsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data = FundingsFormData { funders: vec![] };
        let program_value = "".into();
        let project_name_value = "".into();
        let project_shortname_value = "".into();
        let grant_number_value = "".into();
        let jurisdiction_value = "".into();
        let show_results = false;

        link.send_message(Msg::GetFunders);

        FundingsFormComponent {
            props,
            data,
            program_value,
            project_name_value,
            project_shortname_value,
            grant_number_value,
            jurisdiction_value,
            show_results,
            fetch_funders: Default::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFundersFetchState(fetch_state) => {
                self.fetch_funders.apply(fetch_state);
                self.data.funders = match self.fetch_funders.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.funders.clone(),
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
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchFunder(value) => {
                let body = FundersRequestBody {
                    query: FUNDERS_QUERY.to_string(),
                    variables: Variables {
                        work_id: None,
                        contributor_id: None,
                        limit: None,
                        offset: None,
                        filter: Some(value),
                    },
                };
                let request = FundersRequest { body };
                self.fetch_funders = Fetch::new(request);
                self.link.send_message(Msg::GetFunders);
                false
            }
            Msg::AddFunding(funder) => {
                let mut fundings: Vec<Funding> = self.props.fundings.clone().unwrap_or_default();
                let funder_id = funder.funder_id.clone();
                let funding = Funding {
                    funding_id: "".to_string(),
                    work_id: self.props.work_id.clone(),
                    funder_id,
                    program: None,
                    project_name: None,
                    project_shortname: None,
                    grant_number: None,
                    jurisdiction: None,
                    funder,
                };
                fundings.push(funding);
                self.props.update_fundings.emit(Some(fundings));
                true
            }
            Msg::RemoveFunding(funder_id) => {
                let to_keep: Vec<Funding> = self
                    .props
                    .fundings
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|f| f.funder_id != funder_id)
                    .collect();
                self.props.update_fundings.emit(Some(to_keep));
                true
            }
            Msg::ChangeProgramEditValue(val) => {
                self.program_value.neq_assign(val);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeProgram(funder_id) => {
                let program_value = self.program_value.trim().to_string();
                let program = match program_value.is_empty() {
                    true => None,
                    false => Some(program_value),
                };
                let mut fundings: Vec<Funding> = self.props.fundings.clone().unwrap_or_default();
                if let Some(position) = fundings.iter().position(|f| f.funder_id == funder_id) {
                    let mut funding = fundings[position].clone();
                    funding.program = program;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut fundings[position], funding);
                    self.props.update_fundings.emit(Some(fundings));
                    self.program_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeProjectNameEditValue(val) => {
                self.project_name_value.neq_assign(val);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeProjectName(funder_id) => {
                let project_name_value = self.project_name_value.trim().to_string();
                let project_name = match project_name_value.is_empty() {
                    true => None,
                    false => Some(project_name_value),
                };
                let mut fundings: Vec<Funding> = self.props.fundings.clone().unwrap_or_default();
                if let Some(position) = fundings.iter().position(|f| f.funder_id == funder_id) {
                    let mut funding = fundings[position].clone();
                    funding.project_name = project_name;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut fundings[position], funding);
                    self.props.update_fundings.emit(Some(fundings));
                    self.project_name_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeProjectShortnameEditValue(val) => {
                self.project_shortname_value.neq_assign(val);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeProjectShortname(funder_id) => {
                let project_shortname_value = self.project_shortname_value.trim().to_string();
                let project_shortname = match project_shortname_value.is_empty() {
                    true => None,
                    false => Some(project_shortname_value),
                };
                let mut fundings: Vec<Funding> = self.props.fundings.clone().unwrap_or_default();
                if let Some(position) = fundings.iter().position(|f| f.funder_id == funder_id) {
                    let mut funding = fundings[position].clone();
                    funding.project_shortname = project_shortname;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut fundings[position], funding);
                    self.props.update_fundings.emit(Some(fundings));
                    self.project_shortname_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeGrantEditValue(val) => {
                self.grant_number_value.neq_assign(val);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeGrant(funder_id) => {
                let grant_number_value = self.grant_number_value.trim().to_string();
                let grant_number = match grant_number_value.is_empty() {
                    true => None,
                    false => Some(grant_number_value),
                };
                let mut fundings: Vec<Funding> = self.props.fundings.clone().unwrap_or_default();
                if let Some(position) = fundings.iter().position(|f| f.funder_id == funder_id) {
                    let mut funding = fundings[position].clone();
                    funding.grant_number = grant_number;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut fundings[position], funding);
                    self.props.update_fundings.emit(Some(fundings));
                    self.grant_number_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeJurisdictionEditValue(val) => {
                self.jurisdiction_value.neq_assign(val);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeJurisdiction(funder_id) => {
                let jurisdiction_value = self.jurisdiction_value.trim().to_string();
                let jurisdiction = match jurisdiction_value.is_empty() {
                    true => None,
                    false => Some(jurisdiction_value),
                };
                let mut fundings: Vec<Funding> = self.props.fundings.clone().unwrap_or_default();
                if let Some(position) = fundings.iter().position(|f| f.funder_id == funder_id) {
                    let mut funding = fundings[position].clone();
                    funding.jurisdiction = jurisdiction;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut fundings[position], funding);
                    self.props.update_fundings.emit(Some(fundings));
                    self.jurisdiction_value = "".to_string();
                    true
                } else {
                    false
                }
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let fundings = self.props.fundings.clone().unwrap_or_default();
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
                {
                    if fundings.len() > 0 {
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
    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_funding(&self, f: &Funding) -> Html {
        // there's probably a better way to do this. We basically need to copy 3 instances
        // of funder_id and take ownership of them so they can be passed on to
        // the callback functions
        let funder_id = f.funder_id.clone();
        let pro_fid = f.funder_id.clone();
        let nam_fid = f.funder_id.clone();
        let sna_fid = f.funder_id.clone();
        let grant_fid = f.funder_id.clone();
        let jur_fid = f.funder_id.clone();
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
                    <FormTextInput
                        label="Program"
                        value=&f.program.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeProgramEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeProgram(pro_fid.clone()))
                    />
                    <FormTextInput
                        label="Project Name"
                        value=&f.project_name.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeProjectNameEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeProjectName(nam_fid.clone()))
                    />
                    <FormTextInput
                        label="Project Short Name"
                        value=&f.project_shortname.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeProjectShortnameEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeProjectShortname(sna_fid.clone()))
                    />
                    <FormTextInput
                        label="Grant Number"
                        value=&f.grant_number.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeGrantEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeGrant(grant_fid.clone()))
                    />
                    <FormTextInput
                        label="Jurisdiction"
                        value=&f.jurisdiction.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeJurisdictionEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeJurisdiction(jur_fid.clone()))
                    />
                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::RemoveFunding(funder_id.clone()))
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
