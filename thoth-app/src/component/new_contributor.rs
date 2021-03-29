use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
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
use crate::models::contributor::Contributor;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewContributorComponent {
    contributor: Contributor,
    push_contributor: PushCreateContributor,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
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
    ChangeRoute(AppRoute),
    ToggleDuplicateTooltip(bool),
}

impl Component for NewContributorComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_contributor = Default::default();
        let router = RouteAgentDispatcher::new();
        let notification_bus = NotificationBus::dispatcher();
        let contributor: Contributor = Default::default();
        let show_duplicate_tooltip = false;
        let fetch_contributors = Default::default();
        let contributors = Default::default();

        NewContributorComponent {
            contributor,
            push_contributor,
            link,
            router,
            notification_bus,
            show_duplicate_tooltip,
            fetch_contributors,
            contributors,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Contributor(c.contributor_id.clone()),
                            )));
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
                self.link
                    .send_future(self.push_contributor.fetch(Msg::SetContributorPushState));
                self.link
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
                self.link.send_future(
                    self.fetch_contributors
                        .fetch(Msg::SetContributorsFetchState),
                );
                self.link
                    .send_message(Msg::SetContributorsFetchState(FetchAction::Fetching));
                false
            }
            Msg::ChangeFirstName(value) => {
                let first_name = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.contributor.first_name.neq_assign(first_name)
            }
            Msg::ChangeLastName(last_name) => self
                .contributor
                .last_name
                .neq_assign(last_name.trim().to_owned()),
            Msg::ChangeFullName(full_name) => {
                let body = ContributorsRequestBody {
                    variables: SearchVariables {
                        filter: Some(full_name.clone()),
                        limit: Some(9999),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = ContributorsRequest { body };
                self.fetch_contributors = Fetch::new(request);
                self.link.send_message(Msg::GetContributors);
                self.contributor
                    .full_name
                    .neq_assign(full_name.trim().to_owned())
            }
            Msg::ChangeOrcid(value) => {
                let orcid = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.contributor.orcid.neq_assign(orcid)
            }
            Msg::ChangeWebsite(value) => {
                let website = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.contributor.website.neq_assign(website)
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
            Msg::ToggleDuplicateTooltip(value) => {
                self.show_duplicate_tooltip = value;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let callback = self.link.callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateContributor
        });
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

                <form onsubmit=callback>
                    <FormTextInput
                        label = "Given Name"
                        value=&self.contributor.first_name
                        oninput=self.link.callback(|e: InputData| Msg::ChangeFirstName(e.value))
                    />
                    <FormTextInput
                        label = "Family Name"
                        value=&self.contributor.last_name
                        oninput=self.link.callback(|e: InputData| Msg::ChangeLastName(e.value))
                        required=true
                    />
                    <div class="field">
                        <label class="label">{ "Full Name" }</label>
                        { self.full_name_div() }
                    </div>
                    <FormUrlInput
                        label = "ORCID (Full URL)"
                        value=&self.contributor.orcid
                        oninput=self.link.callback(|e: InputData| Msg::ChangeOrcid(e.value))
                    />
                    <FormUrlInput
                        label = "Website"
                        value=&self.contributor.website
                        oninput=self.link.callback(|e: InputData| Msg::ChangeWebsite(e.value))
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

impl NewContributorComponent {
    fn full_name_div(&self) -> Html {
        let mut tooltip = "Existing contributors with similar names:\n\n".to_string();
        for c in &self.contributors {
            if let Some(orcid) = &c.orcid {
                tooltip.push_str(&format!("{} - {}\n", c.full_name, orcid));
            } else {
                tooltip.push_str(&format!("{}\n", c.full_name));
            }
        }
        match self.show_duplicate_tooltip && !self.contributors.is_empty() {
            true => html! {
                <div
                    class="control is-expanded has-tooltip-arrow has-tooltip-bottom has-tooltip-active"
                    data-tooltip={ tooltip }
                >
                    { self.full_name_input() }
                </div>
            },
            false => html! {
                <div
                    class="control is-expanded"
                >
                    { self.full_name_input() }
                </div>
            },
        }
    }

    fn full_name_input(&self) -> Html {
        html! {
            <input
                class="input"
                input_type="text"
                placeholder="Full Name"
                value=&self.contributor.full_name
                oninput=self.link.callback(|e: InputData| Msg::ChangeFullName(e.value))
                onfocus=self.link.callback(|_| Msg::ToggleDuplicateTooltip(true))
                onblur=self.link.callback(|_| Msg::ToggleDuplicateTooltip(false))
                required=true
            />
        }
    }
}
