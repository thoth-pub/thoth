use gloo_timers::callback::Timeout;
use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::issue::IssueWithSeries;
use thoth_api::model::series::SeriesWithImprint;
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
use crate::component::utils::FormNumberInput;
use crate::models::issue::create_issue_mutation::CreateIssueRequest;
use crate::models::issue::create_issue_mutation::CreateIssueRequestBody;
use crate::models::issue::create_issue_mutation::PushActionCreateIssue;
use crate::models::issue::create_issue_mutation::PushCreateIssue;
use crate::models::issue::create_issue_mutation::Variables as CreateVariables;
use crate::models::issue::delete_issue_mutation::DeleteIssueRequest;
use crate::models::issue::delete_issue_mutation::DeleteIssueRequestBody;
use crate::models::issue::delete_issue_mutation::PushActionDeleteIssue;
use crate::models::issue::delete_issue_mutation::PushDeleteIssue;
use crate::models::issue::delete_issue_mutation::Variables as DeleteVariables;
use crate::models::series::serieses_query::FetchActionSerieses;
use crate::models::series::serieses_query::FetchSerieses;
use crate::models::series::serieses_query::SeriesesRequest;
use crate::models::series::serieses_query::SeriesesRequestBody;
use crate::models::series::serieses_query::Variables;
use crate::models::Dropdown;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_ISSUES;
use crate::string::REMOVE_BUTTON;
use crate::DEFAULT_DEBOUNCING_TIMEOUT;

use super::ToElementValue;

pub struct IssuesFormComponent {
    data: IssuesFormData,
    new_issue: IssueWithSeries,
    show_add_form: bool,
    show_results: bool,
    fetch_serieses: FetchSerieses,
    push_issue: PushCreateIssue,
    delete_issue: PushDeleteIssue,
    notification_bus: NotificationDispatcher,
    // Store props value locally in order to test whether it has been updated on props change
    resource_access: AccountAccess,
    search_callback: Callback<()>,
    search_query: String,
    debounce_timeout: Option<Timeout>,
}

#[derive(Default)]
struct IssuesFormData {
    serieses: Vec<SeriesWithImprint>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetSeriesesFetchState(FetchActionSerieses),
    GetSerieses,
    SetIssuePushState(PushActionCreateIssue),
    CreateIssue,
    SetIssueDeleteState(PushActionDeleteIssue),
    DeleteIssue(Uuid),
    AddIssue(SeriesWithImprint),
    ToggleSearchResultDisplay(bool),
    SearchQueryChanged(String),
    SearchSeries,
    ChangeOrdinal(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub issues: Option<Vec<IssueWithSeries>>,
    pub work_id: Uuid,
    pub imprint_id: Uuid,
    pub current_user: AccountDetails,
    pub update_issues: Callback<Option<Vec<IssueWithSeries>>>,
}

impl Component for IssuesFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let data: IssuesFormData = Default::default();
        let new_issue: IssueWithSeries = Default::default();
        let show_add_form = false;
        let show_results = false;
        let body = SeriesesRequestBody {
            variables: Variables {
                publishers: ctx.props().current_user.resource_access.restricted_to(),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = SeriesesRequest { body };
        let fetch_serieses = Fetch::new(request);
        let push_issue = Default::default();
        let delete_issue = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let resource_access = ctx.props().current_user.resource_access.clone();
        let search_callback = ctx.link().callback(|_| Msg::SearchSeries);
        let search_query: String = Default::default();

        ctx.link().send_message(Msg::GetSerieses);

        IssuesFormComponent {
            data,
            new_issue,
            show_add_form,
            show_results,
            fetch_serieses,
            push_issue,
            delete_issue,
            notification_bus,
            resource_access,
            search_callback,
            search_query,
            debounce_timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
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
                ctx.link()
                    .send_future(self.fetch_serieses.fetch(Msg::SetSeriesesFetchState));
                ctx.link()
                    .send_message(Msg::SetSeriesesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetIssuePushState(fetch_state) => {
                self.push_issue.apply(fetch_state);
                match self.push_issue.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_issue {
                        Some(i) => {
                            let issue = i.clone();
                            let mut issues: Vec<IssueWithSeries> =
                                ctx.props().issues.clone().unwrap_or_default();
                            issues.push(issue);
                            ctx.props().update_issues.emit(Some(issues));
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
            Msg::CreateIssue => {
                let body = CreateIssueRequestBody {
                    variables: CreateVariables {
                        work_id: ctx.props().work_id,
                        series_id: self.new_issue.series_id,
                        issue_ordinal: self.new_issue.issue_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateIssueRequest { body };
                self.push_issue = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_issue.fetch(Msg::SetIssuePushState));
                ctx.link()
                    .send_message(Msg::SetIssuePushState(FetchAction::Fetching));
                false
            }
            Msg::SetIssueDeleteState(fetch_state) => {
                self.delete_issue.apply(fetch_state);
                match self.delete_issue.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_issue {
                        Some(issue) => {
                            let to_keep: Vec<IssueWithSeries> = ctx
                                .props()
                                .issues
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|i| i.issue_id != issue.issue_id)
                                .collect();
                            ctx.props().update_issues.emit(Some(to_keep));
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
            Msg::DeleteIssue(issue_id) => {
                let body = DeleteIssueRequestBody {
                    variables: DeleteVariables { issue_id },
                    ..Default::default()
                };
                let request = DeleteIssueRequest { body };
                self.delete_issue = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_issue.fetch(Msg::SetIssueDeleteState));
                ctx.link()
                    .send_message(Msg::SetIssueDeleteState(FetchAction::Fetching));
                false
            }
            Msg::AddIssue(series) => {
                self.new_issue.series_id = series.series_id;
                self.new_issue.series = series;
                ctx.link().send_message(Msg::ToggleAddFormDisplay(true));
                true
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchQueryChanged(value) => {
                self.search_query = value;
                // cancel previous timeout
                self.debounce_timeout = self.debounce_timeout.take().and_then(|timeout| {
                    timeout.cancel();
                    None
                });
                // start new timeout
                let search_callback = self.search_callback.clone();
                let timeout = Timeout::new(DEFAULT_DEBOUNCING_TIMEOUT, move || {
                    search_callback.emit(());
                });
                self.debounce_timeout = Some(timeout);
                false
            }
            Msg::SearchSeries => {
                let body = SeriesesRequestBody {
                    variables: Variables {
                        filter: Some(self.search_query.clone()),
                        limit: Some(25),
                        publishers: ctx.props().current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = SeriesesRequest { body };
                self.fetch_serieses = Fetch::new(request);
                ctx.link().send_message(Msg::GetSerieses);
                false
            }
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.new_issue.issue_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access.clone());
        if updated_permissions {
            ctx.link().send_message(Msg::GetSerieses);
            false
        } else {
            true
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let issues = ctx.props().issues.clone().unwrap_or_default();
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Issues" }
                </p>
                <div class="panel-block">
                    <div class={ self.search_dropdown_status() } style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Series"
                                        aria-haspopup="true"
                                        aria-controls="serieses-menu"
                                        oninput={ ctx.link().callback(|e: InputEvent| Msg::SearchQueryChanged(e.to_value())) }
                                        onfocus={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(true)) }
                                        onblur={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(false)) }
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
                                        if ctx.props().issues
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .any(|ser| ser.series_id == series.series_id)
                                            // avoid listing series whose imprint doesn't match work
                                            || series.imprint.imprint_id != ctx.props().imprint_id
                                        {
                                            html! {}
                                        } else {
                                            s.as_dropdown_item(
                                                ctx.link().callback(move |_| {
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
                <div class={ self.add_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Issue" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="issues-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateIssue
                            }) }
                            >
                                <div class="field">
                                    <label class="label">{ "Series" }</label>
                                    <div class="control is-expanded">
                                        {&self.new_issue.series.series_name}
                                    </div>
                                </div>
                                <FormNumberInput
                                    label="Issue Ordinal"
                                    value={ self.new_issue.issue_ordinal }
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
                                form="issues-form"
                            >
                                { "Add Issue" }
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
                    if !issues.is_empty() {
                        html!{{for issues.iter().map(|i| self.render_issue(ctx, i))}}
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

    fn render_issue(&self, ctx: &Context<Self>, i: &IssueWithSeries) -> Html {
        let issue_id = i.issue_id;
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
                            {&i.series.issn_print.as_ref().unwrap_or(&String::default())}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "ISSN Digital" }</label>
                        <div class="control is-expanded">
                            {&i.series.issn_print.as_ref().unwrap_or(&String::default())}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Issue Ordinal" }</label>
                        <div class="control is-expanded">
                            {&i.issue_ordinal}
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteIssue(issue_id)) }
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
