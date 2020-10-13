use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::models::issue::Issue;
use crate::models::series::Series;
use crate::models::series::serieses_query::SeriesesRequest;
use crate::models::series::serieses_query::SeriesesRequestBody;
use crate::models::series::serieses_query::FetchActionSerieses;
use crate::models::series::serieses_query::FetchSerieses;
use crate::models::series::serieses_query::Variables;
use crate::models::series::serieses_query::SERIESES_QUERY;
use crate::component::utils::FormNumberInput;
use crate::string::EMPTY_ISSUES;
use crate::string::REMOVE_BUTTON;

pub struct IssuesFormComponent {
    props: Props,
    data: IssuesFormData,
    ordinal_value: i32,
    show_results: bool,
    fetch_serieses: FetchSerieses,
    link: ComponentLink<Self>,
}

struct IssuesFormData {
    serieses: Vec<Series>,
}

pub enum Msg {
    SetSeriesesFetchState(FetchActionSerieses),
    GetSerieses,
    ToggleSearchResultDisplay(bool),
    SearchSeries(String),
    AddIssue(Series),
    RemoveIssue(String),
    ChangeOrdinalEditValue(String),
    ChangeOrdinal(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub issues: Option<Vec<Issue>>,
    pub work_id: String,
    pub update_issues: Callback<Option<Vec<Issue>>>,
}

impl Component for IssuesFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data = IssuesFormData {
            serieses: vec![],
        };
        let ordinal_value = 1;
        let show_results = false;

        link.send_message(Msg::GetSerieses);

        IssuesFormComponent {
            props,
            data,
            ordinal_value,
            show_results,
            fetch_serieses: Default::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetSeriesesFetchState(fetch_state) => {
                self.fetch_serieses.apply(fetch_state);
                self.data.serieses = match self.fetch_serieses.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.serieses.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetSerieses => {
                self.link.send_future(
                    self.fetch_serieses
                        .fetch(Msg::SetSeriesesFetchState),
                );
                self.link
                    .send_message(Msg::SetSeriesesFetchState(FetchAction::Fetching));
                false
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchSeries(value) => {
                let body = SeriesesRequestBody {
                    query: SERIESES_QUERY.to_string(),
                    variables: Variables {
                        work_id: None,
                        contributor_id: None,
                        limit: None,
                        offset: None,
                        filter: Some(value),
                    },
                };
                let request = SeriesesRequest { body };
                self.fetch_serieses = Fetch::new(request);
                self.link.send_message(Msg::GetSerieses);
                false
            }
            Msg::AddIssue(series) => {
                let mut issues: Vec<Issue> =
                    self.props.issues.clone().unwrap_or_default();
                let series_id = series.series_id.clone();
                let issue = Issue {
                    work_id: self.props.work_id.clone(),
                    series_id,
                    issue_ordinal: 1,
                    series,
                };
                issues.push(issue);
                self.props.update_issues.emit(Some(issues));
                true
            }
            Msg::RemoveIssue(series_id) => {
                let to_keep: Vec<Issue> = self
                    .props
                    .issues
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|i| i.series_id != series_id)
                    .collect();
                self.props.update_issues.emit(Some(to_keep));
                true
            }
            Msg::ChangeOrdinalEditValue(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap();
                self.ordinal_value.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeOrdinal(series_id) => {
                let ordinal_value = self.ordinal_value;
                let mut issues: Vec<Issue> =
                    self.props.issues.clone().unwrap_or_default();
                if let Some(position) = issues
                    .iter()
                    .position(|i| i.series_id == series_id)
                {
                    let mut issue = issues[position].clone();
                    issue.issue_ordinal = ordinal_value;
                    // we must acknowledge that replace returns a value, even if we don't want it
                    let _ = std::mem::replace(&mut issues[position], issue);
                    self.props.update_issues.emit(Some(issues));
                    self.ordinal_value = 1;
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
        let issues = self.props.issues.clone().unwrap_or_default();
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Issues" }
                </p>
                <div class="panel-block">
                    <div class=self.search_dropdown_status() style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Series"
                                        aria-haspopup="true"
                                        aria-controls="serieses-menu"
                                        oninput=self.link.callback(|e: InputData| Msg::SearchSeries(e.value))
                                        onfocus=self.link.callback(|_| Msg::ToggleSearchResultDisplay(true))
                                        onblur=self.link.callback(|_| Msg::ToggleSearchResultDisplay(false))
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                            </div>
                        </div>
                        <div class="dropdown-menu" id="serieses-menu" role="menu">
                            <div class="dropdown-content">
                                {
                                    for self.data.serieses.iter().map(|s| {
                                        let series = s.clone();
                                        // avoid listing series already present in issues list
                                        if let Some(_index) = self.props.issues
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .position(|ser| ser.series_id == series.series_id)
                                        {
                                            html! {}
                                        } else {
                                            s.as_dropdown_item(
                                                self.link.callback(move |_| {
                                                    Msg::AddIssue(series.clone())
                                                })
                                            )
                                        }
                                    })
                                }
                            </div>
                        </div>
                    </div>
                </div>
                {
                    if issues.len() > 0 {
                        html!{{for issues.iter().map(|i| self.render_issue(i))}}
                    } else {
                        html! {
                            <div class="notification is-info is-light">
                                { EMPTY_ISSUES }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl IssuesFormComponent {
    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_issue(&self, i: &Issue) -> Html {
        // there's probably a better way to do this. We basically need to copy 3 instances
        // of contributor_id and take ownership of them so they can be passed on to
        // the callback functions
        let series_id = i.series_id.clone();
        let ord_sid = i.series_id.clone();
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="far fa-newspaper" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Series" }</label>
                        <div class="control is-expanded">
                            {&i.series.series_name}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Series Type" }</label>
                        <div class="control is-expanded">
                            {&i.series.series_type}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "ISSN Print" }</label>
                        <div class="control is-expanded">
                            {&i.series.issn_print}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "ISSN Digital" }</label>
                        <div class="control is-expanded">
                            {&i.series.issn_digital}
                        </div>
                    </div>

                    <FormNumberInput
                        label="Issue Ordinal"
                        value=&i.issue_ordinal
                        oninput=self.link.callback(|e: InputData| Msg::ChangeOrdinalEditValue(e.value))
                        onblur=self.link.callback(move |_| Msg::ChangeOrdinal(ord_sid.clone()))
                    />
                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::RemoveIssue(series_id.clone()))
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
