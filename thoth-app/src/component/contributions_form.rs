use std::str::FromStr;
use thoth_api::models::contributor::ContributionType;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::models::contribution::contribution_types_query::FetchActionContributionTypes;
use crate::models::contribution::contribution_types_query::FetchContributionTypes;
use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use crate::models::contributor::contributors_query::CONTRIBUTORS_QUERY;
use crate::models::contribution::Contribution;
use crate::models::contribution::ContributionTypeValues;
use crate::models::contributor::Contributor;
use crate::component::utils::FormBooleanSelect;
use crate::component::utils::FormContributionTypeSelect;
use crate::component::utils::FormTextInput;
use crate::string::EMPTY_CONTRIBUTIONS;
use crate::string::REMOVE_BUTTON;

pub struct ContributionsFormComponent {
    props: Props,
    data: ContributionsFormData,
    institution_value: String,
    biography_value: String,
    contributiontype_value: ContributionType,
    maincontribution_value: bool,
    show_results: bool,
    fetch_contributors: FetchContributors,
    fetch_contribution_types: FetchContributionTypes,
    link: ComponentLink<Self>,
}

struct ContributionsFormData {
    contributors: Vec<Contributor>,
    contribution_types: Vec<ContributionTypeValues>,
}

pub enum Msg {
    SetContributorsFetchState(FetchActionContributors),
    GetContributors,
    SetContributionTypesFetchState(FetchActionContributionTypes),
    GetContributionTypes,
    ToggleSearchResultDisplay(bool),
    SearchContributor(String),
    AddContribution(Contributor),
    RemoveContribution(String),
    ChangeInstitutionEditValue(String),
    ChangeInstitution(String),
    ChangeBiographyEditValue(String),
    ChangeBiography(String),
    ChangeContributiontypeEditValue(ContributionType),
    ChangeContributiontype(String),
    ChangeMainContributionEditValue(bool),
    ChangeMainContribution(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub contributions: Option<Vec<Contribution>>,
    pub work_id: String,
    pub update_contributions: Callback<Option<Vec<Contribution>>>,
}

impl Component for ContributionsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data = ContributionsFormData {
            contributors: vec![],
            contribution_types: vec![],
        };
        let institution_value = "".into();
        let biography_value = "".into();
        let contributiontype_value = ContributionType::Author;
        let maincontribution_value = false;
        let show_results = false;

        link.send_message(Msg::GetContributors);
        link.send_message(Msg::GetContributionTypes);

        ContributionsFormComponent {
            props,
            data,
            institution_value,
            biography_value,
            contributiontype_value,
            maincontribution_value,
            show_results,
            fetch_contributors: Default::default(),
            fetch_contribution_types: Default::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchContributor(value) => {
                let body = ContributorsRequestBody {
                    query: CONTRIBUTORS_QUERY.to_string(),
                    variables: Variables {
                        work_id: None,
                        contributor_id: None,
                        limit: None,
                        offset: None,
                        filter: Some(value),
                    },
                };
                let request = ContributorsRequest { body };
                self.fetch_contributors = Fetch::new(request);
                self.link.send_message(Msg::GetContributors);
                false
            }
            Msg::AddContribution(contributor) => {
                let mut contributions: Vec<Contribution> =
                    self.props.contributions.clone().unwrap_or_default();
                let contributor_id = contributor.contributor_id.clone();
                let contribution = Contribution {
                    work_id: self.props.work_id.clone(),
                    contributor_id: contributor_id.clone(),
                    contribution_type: ContributionType::Author,
                    main_contribution: false,
                    biography: None,
                    institution: None,
                    contributor,
                };
                contributions.push(contribution);
                self.props.update_contributions.emit(Some(contributions));
                true
            }
            Msg::RemoveContribution(contributor_id) => {
                let to_keep: Vec<Contribution> = self
                    .props
                    .contributions
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|c| c.contributor_id != contributor_id)
                    .collect();
                self.props.update_contributions.emit(Some(to_keep));
                true
            }
            Msg::ChangeInstitutionEditValue(institution) => {
                self.institution_value.neq_assign(institution);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeInstitution(contributor_id) => {
                let institution_value = self.institution_value.trim().to_string();
                let institution = match institution_value.is_empty() {
                    true => None,
                    false => Some(institution_value),
                };
                let mut contributions: Vec<Contribution> =
                    self.props.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.institution = institution;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.props.update_contributions.emit(Some(contributions));
                    self.institution_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeBiographyEditValue(biography) => {
                self.biography_value.neq_assign(biography);
                false
            }
            Msg::ChangeBiography(contributor_id) => {
                let biography_value = self.biography_value.trim().to_string();
                let biography = match biography_value.is_empty() {
                    true => None,
                    false => Some(biography_value),
                };
                let mut contributions: Vec<Contribution> =
                    self.props.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.biography = biography;
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.props.update_contributions.emit(Some(contributions));
                    self.biography_value = "".to_string();
                    true
                } else {
                    false
                }
            }
            Msg::ChangeContributiontype(contributor_id) => {
                let mut contributions: Vec<Contribution> =
                    self.props.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.contribution_type = self.contributiontype_value.clone();
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.props.update_contributions.emit(Some(contributions));
                    self.contributiontype_value = ContributionType::Author;
                    true
                } else {
                    false
                }
            }
            Msg::ChangeContributiontypeEditValue(contribution_type) => {
                self.contributiontype_value.neq_assign(contribution_type)
            }
            Msg::ChangeMainContribution(contributor_id) => {
                let mut contributions: Vec<Contribution> =
                    self.props.contributions.clone().unwrap_or_default();
                if let Some(position) = contributions
                    .iter()
                    .position(|c| c.contributor_id == contributor_id)
                {
                    let mut contribution = contributions[position].clone();
                    contribution.main_contribution = self.maincontribution_value.clone();
                    let _ = std::mem::replace(&mut contributions[position], contribution);
                    self.props.update_contributions.emit(Some(contributions));
                    self.maincontribution_value = false;
                    true
                } else {
                    false
                }
            }
            Msg::ChangeMainContributionEditValue(main_contribution) => {
                self.maincontribution_value.neq_assign(main_contribution)
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let contributions = self.props.contributions.clone().unwrap_or_else(|| vec![]);
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
                                { for self.data.contributors.iter().map(|c| self.render_contributors(c)) }
                            </div>
                        </div>
                    </div>
                </div>
                {
                    if contributions.len() > 0 {
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
    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_contributors(&self, c: &Contributor) -> Html {
        let contributor = c.clone();
        // avoid listing contributors already present in contributions list
        if let Some(_index) = self.props.contributions
            .as_ref()
            .unwrap()
            .iter()
            .position(|ctr| ctr.contributor_id == c.contributor_id)
        {
            html! {}
        } else {
            // since contributors dropdown has an onblur event, we need to use onmousedown instead of
            // onclick. This is not ideal, but it seems to be the only event that'd do the calback
            // without disabling onblur so that onclick can take effect
            html! {
                <div
                    onmousedown=self.link.callback(move |_| Msg::AddContribution(contributor.clone()))
                    class="dropdown-item"
                >
                {
                    if let Some(orcid) = &c.orcid {
                        format!("{} - {}", &c.full_name, orcid)
                    } else {
                        format!("{}", &c.full_name )
                    }
                }
                </div>
            }
        }
    }

    fn render_contribution(&self, c: &Contribution) -> Html {
        // there's probably a better way to do this. We basically need to copy 3 instances
        // of contributor_id and take ownership of them so they can be passed on to
        // the callback functions
        let contributor_id = c.contributor_id.clone();
        let type_cid = c.contributor_id.clone();
        let inst_cid = c.contributor_id.clone();
        let bio_cid = c.contributor_id.clone();
        let main_cid = c.contributor_id.clone();
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-user" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Full Name" }</label>
                        <div class="control is-expanded">
                            {&c.contributor.full_name}
                        </div>
                    </div>
                    <FormContributionTypeSelect
                        label = "Contribution Type"
                        value=&c.contribution_type
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                Msg::ChangeContributiontypeEditValue(ContributionType::from_str(&value).unwrap())
                            }
                            _ => unreachable!(),
                        })
                        onblur=self.link.callback(move |_| Msg::ChangeContributiontype(type_cid.clone()))

                        data=&self.data.contribution_types
                        required = true
                    />
                    <FormTextInput
                        label="Institution"
                        value=&c.institution.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeInstitutionEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeInstitution(inst_cid.clone()))
                    />
                    <FormTextInput
                        label="Biography"
                        value=&c.biography.clone().unwrap_or_else(|| "".to_string())
                        oninput=self.link.callback(|e: InputData| Msg::ChangeBiographyEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeBiography(bio_cid.clone()))
                    />
                    <FormBooleanSelect
                        label = "Main"
                        value=c.main_contribution
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                let boolean = value == "true".to_string();
                                Msg::ChangeMainContributionEditValue(boolean)
                            }
                            _ => unreachable!(),
                        })
                        onblur=self.link.callback(move |_| Msg::ChangeMainContribution(main_cid.clone()))
                        required = true
                    />
                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::RemoveContribution(contributor_id.clone()))
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
