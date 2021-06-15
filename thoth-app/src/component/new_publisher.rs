use thoth_api::publisher::model::Publisher;
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
use crate::models::publisher::create_publisher_mutation::CreatePublisherRequest;
use crate::models::publisher::create_publisher_mutation::CreatePublisherRequestBody;
use crate::models::publisher::create_publisher_mutation::PushActionCreatePublisher;
use crate::models::publisher::create_publisher_mutation::PushCreatePublisher;
use crate::models::publisher::create_publisher_mutation::Variables;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewPublisherComponent {
    publisher: Publisher,
    push_publisher: PushCreatePublisher,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetPublisherPushState(PushActionCreatePublisher),
    CreatePublisher,
    ChangePublisherName(String),
    ChangePublisherShortname(String),
    ChangePublisherUrl(String),
    ChangeRoute(AppRoute),
}

impl Component for NewPublisherComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_publisher = Default::default();
        let router = RouteAgentDispatcher::new();
        let notification_bus = NotificationBus::dispatcher();
        let publisher: Publisher = Default::default();

        NewPublisherComponent {
            publisher,
            push_publisher,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetPublisherPushState(fetch_state) => {
                self.push_publisher.apply(fetch_state);
                match self.push_publisher.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_publisher {
                        Some(p) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", p.publisher_name),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Publisher(p.publisher_id),
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
            Msg::CreatePublisher => {
                let body = CreatePublisherRequestBody {
                    variables: Variables {
                        publisher_name: self.publisher.publisher_name.clone(),
                        publisher_shortname: self.publisher.publisher_shortname.clone(),
                        publisher_url: self.publisher.publisher_url.clone(),
                    },
                    ..Default::default()
                };
                let request = CreatePublisherRequest { body };
                self.push_publisher = Fetch::new(request);
                self.link
                    .send_future(self.push_publisher.fetch(Msg::SetPublisherPushState));
                self.link
                    .send_message(Msg::SetPublisherPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublisherName(publisher_name) => self
                .publisher
                .publisher_name
                .neq_assign(publisher_name.trim().to_owned()),
            Msg::ChangePublisherShortname(value) => {
                let publisher_shortname = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.publisher
                    .publisher_shortname
                    .neq_assign(publisher_shortname)
            }
            Msg::ChangePublisherUrl(value) => {
                let publisher_url = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.publisher.publisher_url.neq_assign(publisher_url)
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
        let callback = self.link.callback(|event: FocusEvent| {
            event.prevent_default();
            Msg::CreatePublisher
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New publisher" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit=callback>
                    <FormTextInput
                        label = "Publisher Name"
                        value=self.publisher.publisher_name.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangePublisherName(e.value))
                        required=true
                    />
                    <FormTextInput
                        label = "Publisher Short Name"
                        value=self.publisher.publisher_shortname.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangePublisherShortname(e.value))
                    />
                    <FormUrlInput
                        label = "Publisher URL"
                        value=self.publisher.publisher_url.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangePublisherUrl(e.value))
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
