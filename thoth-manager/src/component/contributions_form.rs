use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::api::contributors_query::CONTRIBUTORS_QUERY;
use crate::api::contributors_query::ContributorsRequest;
use crate::api::contributors_query::ContributorsRequestBody;
use crate::api::contributors_query::FetchActionContributors;
use crate::api::contributors_query::FetchContributors;
use crate::api::contributors_query::Variables;
use crate::api::contribution_types_query::FetchActionContributionTypes;
use crate::api::contribution_types_query::FetchContributionTypes;
use crate::api::models::Contribution;
use crate::api::models::Contributor;
use crate::api::models::ContributionTypeValues;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormContributionTypeSelect;

pub struct ContributionsFormComponent {
    contributions: Vec<Contribution>,
    data: ContributionsFormData,
    show_results: bool,
    fetch_contributors: FetchContributors,
    fetch_contribution_types: FetchContributionTypes,
    link: ComponentLink<Self>,
}

struct ContributionsFormData {
    contributors: Vec<Contributor>,
    contribution_types: Vec<ContributionTypeValues>
}

pub enum Msg {
    SetContributorsFetchState(FetchActionContributors),
    GetContributors,
    SetContributionTypesFetchState(FetchActionContributionTypes),
    GetContributionTypes,
    ToggleSearchResultDisplay(bool),
    SearchContributor(String),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub contributions: Vec<Contribution>,
}

impl Component for ContributionsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let contributions = props.contributions;
        let data = ContributionsFormData {
            contributors: vec![],
            contribution_types: vec![],
        };
        let show_results = false;

        link.send_message(Msg::GetContributors);
        link.send_message(Msg::GetContributionTypes);

        ContributionsFormComponent {
            contributions,
            data,
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
                self.link
                    .send_future(self.fetch_contributors.fetch(Msg::SetContributorsFetchState));
                self.link
                    .send_message(Msg::SetContributorsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetContributionTypesFetchState(fetch_state) => {
                self.fetch_contribution_types.apply(fetch_state);
                self.data.contribution_types = match self.fetch_contribution_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.contribution_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetContributionTypes => {
                self.link
                    .send_future(self.fetch_contribution_types.fetch(Msg::SetContributionTypesFetchState));
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
                        filter: Some(value),
                    },
                };
                let request = ContributorsRequest { body };
                self.fetch_contributors = Fetch::new(request);
                self.link.send_message(Msg::GetContributors);
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
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
                                    for self.data.contributors.iter().map(|c| html!{
                                        <a href="#" class="dropdown-item">{ &c.full_name }</a>
                                    })
                                }
                            </div>
                        </div>
                    </div>
                </div>
                { for self.contributions.iter().map(|c| self.render_contribution(c)) }
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

    fn render_contribution(&self, c: &Contribution) -> Html {
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
                        data=&self.data.contribution_types
                        required = true
                    />
                    <FormTextInput
                        label="Institution"
                        value=&c.institution.clone().unwrap_or_else(|| "".to_string())
                    />
                    <FormTextInput
                        label="Biography"
                        value=&c.biography.clone().unwrap_or_else(|| "".to_string())
                    />
                    <div class="field">
                        <label class="label">{ "Main" }</label>
                        <div class="control is-expanded">
                            <input type="checkbox" checked={c.main_contribution} />
                        </div>
                    </div>
                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <button class="button is-danger">
                                { "Remove" }
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
