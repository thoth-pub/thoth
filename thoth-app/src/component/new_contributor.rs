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
use crate::models::contributor::create_contributor_mutation::CreateContributorRequest;
use crate::models::contributor::create_contributor_mutation::CreateContributorRequestBody;
use crate::models::contributor::create_contributor_mutation::PushActionCreateContributor;
use crate::models::contributor::create_contributor_mutation::PushCreateContributor;
use crate::models::contributor::create_contributor_mutation::Variables;
use crate::models::contributor::Contributor;
use crate::string::SAVE_BUTTON;

pub struct NewContributorComponent {
    contributor: Contributor,
    push_contributor: PushCreateContributor,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetContributorPushState(PushActionCreateContributor),
    CreateContributor,
    ChangeFirstName(String),
    ChangeLastName(String),
    ChangeFullName(String),
    ChangeOrcid(String),
    ChangeWebsite(String),
}

impl Component for NewContributorComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_contributor = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let contributor: Contributor = Default::default();

        NewContributorComponent {
            contributor,
            push_contributor,
            link,
            notification_bus,
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
        let callback = self.link.callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreateContributor
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
                    required=true
                />
                <FormTextInput
                    label = "Full Name"
                    value=&self.contributor.full_name
                    oninput=self.link.callback(|e: InputData| Msg::ChangeFullName(e.value))
                    required=true
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
}
