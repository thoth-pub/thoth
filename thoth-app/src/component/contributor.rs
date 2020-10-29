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
use crate::component::utils::FormUrlInput;
use crate::component::utils::Loader;
use crate::models::contributor::contributor_query::ContributorRequest;
use crate::models::contributor::contributor_query::ContributorRequestBody;
use crate::models::contributor::contributor_query::FetchActionContributor;
use crate::models::contributor::contributor_query::FetchContributor;
use crate::models::contributor::contributor_query::Variables;
use crate::models::contributor::update_contributor_mutation::PushActionUpdateContributor;
use crate::models::contributor::update_contributor_mutation::PushUpdateContributor;
use crate::models::contributor::update_contributor_mutation::UpdateContributorRequest;
use crate::models::contributor::update_contributor_mutation::UpdateContributorRequestBody;
use crate::models::contributor::update_contributor_mutation::Variables as UpdateVariables;
use crate::models::contributor::Contributor;
use crate::string::SAVE_BUTTON;

pub struct ContributorComponent {
    contributor: Contributor,
    fetch_contributor: FetchContributor,
    push_contributor: PushUpdateContributor,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetContributorFetchState(FetchActionContributor),
    GetContributor,
    SetContributorPushState(PushActionUpdateContributor),
    UpdateContributor,
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeOrcid(String),
    ChangeWebsite(String),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub contributor_id: String,
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
        let notification_bus = NotificationBus::dispatcher();
        let contributor: Contributor = Default::default();

        link.send_message(Msg::GetContributor);

        ContributorComponent {
            contributor,
            fetch_contributor,
            push_contributor,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
                        contributor_id: self.contributor.contributor_id.clone(),
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
            Msg::ChangeFirstName(first_name) => {
                self.contributor.first_name.neq_assign(Some(first_name))
            }
            Msg::ChangeLastName(last_name) => self.contributor.last_name.neq_assign(last_name),
            Msg::ChangeFullName(full_name) => self.contributor.full_name.neq_assign(full_name),
            Msg::ChangeOrcid(orcid) => self.contributor.orcid.neq_assign(Some(orcid)),
            Msg::ChangeWebsite(website) => self.contributor.website.neq_assign(Some(website)),
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
                    <form onsubmit=callback>
                        <FormTextInput
                            label = "First Name"
                            value=&self.contributor.first_name
                            oninput=self.link.callback(|e: InputData| Msg::ChangeFirstName(e.value))
                        />
                        <FormTextInput
                            label = "Last Name"
                            value=&self.contributor.last_name
                            oninput=self.link.callback(|e: InputData| Msg::ChangeLastName(e.value))
                        />
                        <FormTextInput
                            label = "Full Name"
                            value=&self.contributor.full_name
                            oninput=self.link.callback(|e: InputData| Msg::ChangeFullName(e.value))
                            required = true
                        />
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
                }
            }
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
