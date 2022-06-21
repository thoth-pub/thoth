use thoth_api::model::contributor::Contributor;
use thoth_api::model::{Orcid, ORCID_DOMAIN};
use thoth_errors::ThothError;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yew_router::history::History;
use yew_router::prelude::RouterScopeExt;
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
use crate::component::utils::FormTextInputExtended;
use crate::component::utils::FormUrlInput;
use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables as SearchVariables;
use crate::models::contributor::create_contributor_mutation::CreateContributorRequest;
use crate::models::contributor::create_contributor_mutation::CreateContributorRequestBody;
use crate::models::contributor::create_contributor_mutation::PushActionCreateContributor;
use crate::models::contributor::create_contributor_mutation::PushCreateContributor;
use crate::models::contributor::create_contributor_mutation::Variables;
use crate::models::EditRoute;
use crate::string::SAVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

// Account for possibility of e.g. Chinese full names with only 2 characters.
const MIN_FULLNAME_LEN: usize = 2;

pub struct NewContributorComponent {
    contributor: Contributor,
    // Track the user-entered ORCID string, which may not be validly formatted
    orcid: String,
    orcid_warning: String,
    push_contributor: PushCreateContributor,
    notification_bus: NotificationDispatcher,
    show_duplicate_tooltip: bool,
    fetch_contributors: FetchContributors,
    contributors: Vec<Contributor>,
}

pub enum Msg {
    SetContributorPushState(PushActionCreateContributor),
    CreateContributor,
    SetContributorsFetchState(FetchActionContributors),
    GetContributors,
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeOrcid(String),
    ChangeWebsite(String),
    ToggleDuplicateTooltip(bool),
}

impl Component for NewContributorComponent {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let push_contributor = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let contributor: Contributor = Default::default();
        let orcid = Default::default();
        let orcid_warning = Default::default();
        let show_duplicate_tooltip = false;
        let fetch_contributors = Default::default();
        let contributors = Default::default();

        NewContributorComponent {
            contributor,
            orcid,
            orcid_warning,
            push_contributor,
            notification_bus,
            show_duplicate_tooltip,
            fetch_contributors,
            contributors,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetContributorPushState(fetch_state) => {
                self.push_contributor.apply(fetch_state);
                match self.push_contributor.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_contributor {
                        Some(c) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", c.full_name),
                                NotificationStatus::Success,
                            )));
                            ctx.link().history().unwrap().push(c.edit_route());
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
            Msg::CreateContributor => {
                // Only update the ORCID value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
                // If no ORCID was provided, no format check is required.
                if self.orcid.is_empty() {
                    self.contributor.orcid.neq_assign(None);
                } else if let Ok(result) = self.orcid.parse::<Orcid>() {
                    self.contributor.orcid.neq_assign(Some(result));
                }
                let body = CreateContributorRequestBody {
                    variables: Variables {
                        first_name: self.contributor.first_name.clone(),
                        last_name: self.contributor.last_name.clone(),
                        full_name: self.contributor.full_name.clone(),
                        orcid: self.contributor.orcid.clone(),
                        website: self.contributor.website.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateContributorRequest { body };
                self.push_contributor = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_contributor.fetch(Msg::SetContributorPushState));
                ctx.link()
                    .send_message(Msg::SetContributorPushState(FetchAction::Fetching));
                false
            }
            Msg::SetContributorsFetchState(fetch_state) => {
                self.fetch_contributors.apply(fetch_state);
                self.contributors = match self.fetch_contributors.as_ref().state() {
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
            Msg::ChangeFirstName(value) => self
                .contributor
                .first_name
                .neq_assign(value.to_opt_string()),
            Msg::ChangeLastName(last_name) => self
                .contributor
                .last_name
                .neq_assign(last_name.trim().to_owned()),
            Msg::ChangeFullName(full_name) => {
                if self
                    .contributor
                    .full_name
                    .neq_assign(full_name.trim().to_owned())
                {
                    if self.contributor.full_name.len() < MIN_FULLNAME_LEN {
                        // Don't show similar names tooltip - name too short.
                        self.contributors.clear();
                        true
                    } else {
                        // Search for similar existing names to show in tooltip.
                        let body = ContributorsRequestBody {
                            variables: SearchVariables {
                                filter: Some(self.contributor.full_name.clone()),
                                limit: Some(30),
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        let request = ContributorsRequest { body };
                        self.fetch_contributors = Fetch::new(request);
                        ctx.link().send_message(Msg::GetContributors);
                        // Don't need to re-render here, as another re-render will be
                        // triggered when the message query response is received.
                        false
                    }
                } else {
                    false
                }
            }
            Msg::ChangeOrcid(value) => {
                if self.orcid.neq_assign(value.trim().to_owned()) {
                    // If ORCID is not correctly formatted, display a warning.
                    // Don't update self.contributor.orcid yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.orcid_warning.clear();
                    match self.orcid.parse::<Orcid>() {
                        Err(e) => {
                            match e {
                                // If no ORCID was provided, no warning is required.
                                ThothError::OrcidEmptyError => {}
                                _ => self.orcid_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.orcid = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeWebsite(value) => self.contributor.website.neq_assign(value.to_opt_string()),
            Msg::ToggleDuplicateTooltip(value) => {
                self.show_duplicate_tooltip = value;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateContributor
        });
        let mut tooltip = String::new();
        if self.show_duplicate_tooltip && !self.contributors.is_empty() {
            tooltip = "Existing contributors with similar names:\n\n".to_string();
            for c in &self.contributors {
                tooltip = format!("{}{}\n", tooltip, c);
            }
        }
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New contributor" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit={ callback }>
                    <FormTextInput
                        label = "Given Name"
                        value={ self.contributor.first_name.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFirstName(e.to_value())) }
                    />
                    <FormTextInput
                        label = "Family Name"
                        value={ self.contributor.last_name.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLastName(e.to_value())) }
                        required = true
                    />
                    <FormTextInputExtended
                        label = "Full Name"
                        value={ self.contributor.full_name.clone() }
                        tooltip={ tooltip }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFullName(e.to_value())) }
                        onfocus={ ctx.link().callback(|_| Msg::ToggleDuplicateTooltip(true)) }
                        onblur={ ctx.link().callback(|_| Msg::ToggleDuplicateTooltip(false)) }
                        required = true
                    />
                    <FormTextInputExtended
                        label = "ORCID"
                        statictext={ ORCID_DOMAIN }
                        value={ self.orcid.clone() }
                        tooltip={ self.orcid_warning.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOrcid(e.to_value())) }
                    />
                    <FormUrlInput
                        label = "Website"
                        value={ self.contributor.website.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeWebsite(e.to_value())) }
                    />

                    <div class="field">
                        <div class="control">
                            <button class="button is-success" type="submit">
                                { SAVE_BUTTON }
                            </button>
                        </div>
                    </div>
                </form>
            </>
        }
    }
}
