use thoth_api::contribution::model::SlimContribution;
use thoth_api::contributor::model::Contributor;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::prelude::RouterAnchor;
use yew_router::route::Route;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::contributor_activity_checker::ContributorActivityChecker;
use crate::agent::contributor_activity_checker::Request as ContributorActivityRequest;
use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::delete_dialogue::ConfirmDeleteComponent;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::component::utils::Loader;
use crate::models::contributor::contributor_activity_query::ContributorActivityResponseData;
use crate::models::contributor::contributor_query::ContributorRequest;
use crate::models::contributor::contributor_query::ContributorRequestBody;
use crate::models::contributor::contributor_query::FetchActionContributor;
use crate::models::contributor::contributor_query::FetchContributor;
use crate::models::contributor::contributor_query::Variables;
use crate::models::contributor::delete_contributor_mutation::DeleteContributorRequest;
use crate::models::contributor::delete_contributor_mutation::DeleteContributorRequestBody;
use crate::models::contributor::delete_contributor_mutation::PushActionDeleteContributor;
use crate::models::contributor::delete_contributor_mutation::PushDeleteContributor;
use crate::models::contributor::delete_contributor_mutation::Variables as DeleteVariables;
use crate::models::contributor::update_contributor_mutation::PushActionUpdateContributor;
use crate::models::contributor::update_contributor_mutation::PushUpdateContributor;
use crate::models::contributor::update_contributor_mutation::UpdateContributorRequest;
use crate::models::contributor::update_contributor_mutation::UpdateContributorRequestBody;
use crate::models::contributor::update_contributor_mutation::Variables as UpdateVariables;
use crate::models::EditRoute;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct ContributorComponent {
    contributor: Contributor,
    fetch_contributor: FetchContributor,
    push_contributor: PushUpdateContributor,
    delete_contributor: PushDeleteContributor,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
    _contributor_activity_checker: Box<dyn Bridge<ContributorActivityChecker>>,
    contributor_activity: Vec<SlimContribution>,
}

pub enum Msg {
    GetContributorActivity(ContributorActivityResponseData),
    SetContributorFetchState(FetchActionContributor),
    GetContributor,
    SetContributorPushState(PushActionUpdateContributor),
    UpdateContributor,
    SetContributorDeleteState(PushActionDeleteContributor),
    DeleteContributor,
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeOrcid(String),
    ChangeWebsite(String),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub contributor_id: Uuid,
}

impl Component for ContributorComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let body = ContributorRequestBody {
            variables: Variables {
                contributor_id: Some(props.contributor_id),
            },
            ..Default::default()
        };
        let request = ContributorRequest { body };
        let fetch_contributor = Fetch::new(request);
        let push_contributor = Default::default();
        let delete_contributor = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let contributor: Contributor = Default::default();
        let router = RouteAgentDispatcher::new();
        let mut _contributor_activity_checker =
            ContributorActivityChecker::bridge(link.callback(Msg::GetContributorActivity));
        let contributor_activity = Default::default();

        link.send_message(Msg::GetContributor);
        _contributor_activity_checker.send(
            ContributorActivityRequest::RetrieveContributorActivity(props.contributor_id),
        );

        ContributorComponent {
            contributor,
            fetch_contributor,
            push_contributor,
            delete_contributor,
            link,
            router,
            notification_bus,
            _contributor_activity_checker,
            contributor_activity,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetContributorActivity(response) => {
                let mut should_render = false;
                if let Some(contributor) = response.contributor {
                    if let Some(contributions) = contributor.contributions {
                        if !contributions.is_empty() {
                            self.contributor_activity = contributions;
                            should_render = true;
                        }
                    }
                }
                should_render
            }
            Msg::SetContributorFetchState(fetch_state) => {
                self.fetch_contributor.apply(fetch_state);
                match self.fetch_contributor.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        self.contributor = match &body.data.contributor {
                            Some(c) => c.to_owned(),
                            None => Default::default(),
                        };
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetContributor => {
                self.link
                    .send_future(self.fetch_contributor.fetch(Msg::SetContributorFetchState));
                self.link
                    .send_message(Msg::SetContributorFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetContributorPushState(fetch_state) => {
                self.push_contributor.apply(fetch_state);
                match self.push_contributor.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_contributor {
                        Some(c) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", c.full_name),
                                NotificationStatus::Success,
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
            Msg::UpdateContributor => {
                let body = UpdateContributorRequestBody {
                    variables: UpdateVariables {
                        contributor_id: self.contributor.contributor_id,
                        first_name: self.contributor.first_name.clone(),
                        last_name: self.contributor.last_name.clone(),
                        full_name: self.contributor.full_name.clone(),
                        orcid: self.contributor.orcid.clone(),
                        website: self.contributor.website.clone(),
                    },
                    ..Default::default()
                };
                let request = UpdateContributorRequest { body };
                self.push_contributor = Fetch::new(request);
                self.link
                    .send_future(self.push_contributor.fetch(Msg::SetContributorPushState));
                self.link
                    .send_message(Msg::SetContributorPushState(FetchAction::Fetching));
                false
            }
            Msg::SetContributorDeleteState(fetch_state) => {
                self.delete_contributor.apply(fetch_state);
                match self.delete_contributor.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_contributor {
                        Some(c) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Deleted {}", c.full_name),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Contributors,
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
            Msg::DeleteContributor => {
                let body = DeleteContributorRequestBody {
                    variables: DeleteVariables {
                        contributor_id: self.contributor.contributor_id,
                    },
                    ..Default::default()
                };
                let request = DeleteContributorRequest { body };
                self.delete_contributor = Fetch::new(request);
                self.link.send_future(
                    self.delete_contributor
                        .fetch(Msg::SetContributorDeleteState),
                );
                self.link
                    .send_message(Msg::SetContributorDeleteState(FetchAction::Fetching));
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
            Msg::ChangeFullName(full_name) => self
                .contributor
                .full_name
                .neq_assign(full_name.trim().to_owned()),
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
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match self.fetch_contributor.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = self.link.callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::UpdateContributor
                });
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit contributor" }
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                    <ConfirmDeleteComponent
                                        onclick=self.link.callback(|_| Msg::DeleteContributor)
                                        object_name=self.contributor.full_name.clone()
                                    />
                                </p>
                            </div>
                        </nav>

                        { if !self.contributor_activity.is_empty() {
                            html! {
                                <div class="notification is-link">
                                    {
                                        for self.contributor_activity.iter().map(|contribution| {
                                            html! {
                                                <p>
                                                    { "Contributed to: " }
                                                    <RouterAnchor<AppRoute>
                                                        route=contribution.work.edit_route()
                                                    >
                                                        { &contribution.work.title }
                                                    </  RouterAnchor<AppRoute>>
                                                    { format!(", from: {}", contribution.work.imprint.publisher.publisher_name) }
                                                </p>
                                            }
                                        })
                                    }
                                </div>
                                }
                            } else {
                                html! {}
                            }
                        }

                        <form onsubmit=callback>
                            <FormTextInput
                                label = "Given Name"
                                value=self.contributor.first_name.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeFirstName(e.value))
                            />
                            <FormTextInput
                                label = "Family Name"
                                value=self.contributor.last_name.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeLastName(e.value))
                            />
                            <FormTextInput
                                label = "Full Name"
                                value=self.contributor.full_name.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeFullName(e.value))
                                required = true
                            />
                            <FormUrlInput
                                label = "ORCID (Full URL)"
                                value=self.contributor.orcid.clone()
                                oninput=self.link.callback(|e: InputData| Msg::ChangeOrcid(e.value))
                            />
                            <FormUrlInput
                                label = "Website"
                                value=self.contributor.website.clone()
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
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
