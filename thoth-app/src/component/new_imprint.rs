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
use crate::component::utils::FormPublisherSelect;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::models::imprint::create_imprint_mutation::CreateImprintRequest;
use crate::models::imprint::create_imprint_mutation::CreateImprintRequestBody;
use crate::models::imprint::create_imprint_mutation::PushActionCreateImprint;
use crate::models::imprint::create_imprint_mutation::PushCreateImprint;
use crate::models::imprint::create_imprint_mutation::Variables;
use crate::models::imprint::Imprint;
use crate::models::publisher::publishers_query::FetchActionPublishers;
use crate::models::publisher::publishers_query::FetchPublishers;
use crate::models::publisher::Publisher;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewImprintComponent {
    imprint: Imprint,
    publisher_id: String,
    push_imprint: PushCreateImprint,
    data: ImprintFormData,
    fetch_publishers: FetchPublishers,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct ImprintFormData {
    publishers: Vec<Publisher>,
}

pub enum Msg {
    SetPublishersFetchState(FetchActionPublishers),
    GetPublishers,
    SetImprintPushState(PushActionCreateImprint),
    CreateImprint,
    ChangePublisher(String),
    ChangeImprintName(String),
    ChangeImprintUrl(String),
    ChangeRoute(AppRoute),
}

impl Component for NewImprintComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_imprint = Default::default();
        let router = RouteAgentDispatcher::new();
        let notification_bus = NotificationBus::dispatcher();
        let imprint: Imprint = Default::default();
        let publisher_id: String = Default::default();
        let data: ImprintFormData = Default::default();
        let fetch_publishers: FetchPublishers = Default::default();

        link.send_message(Msg::GetPublishers);

        NewImprintComponent {
            imprint,
            publisher_id,
            push_imprint,
            data,
            fetch_publishers,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetPublishersFetchState(fetch_state) => {
                self.fetch_publishers.apply(fetch_state);
                self.data.publishers = match self.fetch_publishers.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.publishers.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetPublishers => {
                self.link
                    .send_future(self.fetch_publishers.fetch(Msg::SetPublishersFetchState));
                self.link
                    .send_message(Msg::SetPublishersFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetImprintPushState(fetch_state) => {
                self.push_imprint.apply(fetch_state);
                match self.push_imprint.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_imprint {
                        Some(i) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", i.imprint_name),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Imprint(i.imprint_id.clone()),
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
            Msg::CreateImprint => {
                let body = CreateImprintRequestBody {
                    variables: Variables {
                        imprint_name: self.imprint.imprint_name.clone(),
                        imprint_url: self.imprint.imprint_url.clone(),
                        publisher_id: self.publisher_id.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateImprintRequest { body };
                self.push_imprint = Fetch::new(request);
                self.link
                    .send_future(self.push_imprint.fetch(Msg::SetImprintPushState));
                self.link
                    .send_message(Msg::SetImprintPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublisher(publisher_id) => self.publisher_id.neq_assign(publisher_id),
            Msg::ChangeImprintName(imprint_name) => {
                self.imprint.imprint_name.neq_assign(imprint_name)
            }
            Msg::ChangeImprintUrl(imprint_url) => {
                self.imprint.imprint_url.neq_assign(Some(imprint_url))
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
            Msg::CreateImprint
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New imprint" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit=callback>
                    <FormPublisherSelect
                        label = "Publisher"
                        value=&self.publisher_id
                        data=&self.data.publishers
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                Msg::ChangePublisher(value.clone())
                            }
                            _ => unreachable!(),
                        })
                        required = true
                    />
                    <FormTextInput
                        label = "Imprint Name"
                        value=&self.imprint.imprint_name
                        oninput=self.link.callback(|e: InputData| Msg::ChangeImprintName(e.value))
                        required=true
                    />
                    <FormUrlInput
                        label = "Imprint URL"
                        value=&self.imprint.imprint_url
                        oninput=self.link.callback(|e: InputData| Msg::ChangeImprintUrl(e.value))
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
