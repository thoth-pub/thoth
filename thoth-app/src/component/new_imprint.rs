use thoth_api::account::model::AccountDetails;
use thoth_api::imprint::model::ImprintExtended;
use thoth_api::publisher::model::Publisher;
use uuid::Uuid;
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
use crate::models::publisher::publishers_query::FetchActionPublishers;
use crate::models::publisher::publishers_query::FetchPublishers;
use crate::models::publisher::publishers_query::PublishersRequest;
use crate::models::publisher::publishers_query::PublishersRequestBody;
use crate::models::publisher::publishers_query::Variables as PublishersVariables;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewImprintComponent {
    imprint: ImprintExtended,
    publisher_id: Uuid,
    push_imprint: PushCreateImprint,
    data: ImprintFormData,
    fetch_publishers: FetchPublishers,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
    props: Props,
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
    ChangePublisher(Uuid),
    ChangeImprintName(String),
    ChangeImprintUrl(String),
    ChangeRoute(AppRoute),
}
#[derive(Clone, Properties)]
pub struct Props {
    pub current_user: AccountDetails,
}

impl Component for NewImprintComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_imprint = Default::default();
        let router = RouteAgentDispatcher::new();
        let notification_bus = NotificationBus::dispatcher();
        let imprint: ImprintExtended = Default::default();
        let publisher_id: Uuid = Default::default();
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
            props,
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
                let body = PublishersRequestBody {
                    variables: PublishersVariables {
                        publishers: self.props.current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = PublishersRequest { body };
                self.fetch_publishers = Fetch::new(request);

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
                            self.link.send_message(Msg::ChangeRoute(i.edit_route()));
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
                        publisher_id: self.publisher_id,
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
            Msg::ChangeImprintName(imprint_name) => self
                .imprint
                .imprint_name
                .neq_assign(imprint_name.trim().to_owned()),
            Msg::ChangeImprintUrl(value) => {
                let imprint_url = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.imprint.imprint_url.neq_assign(imprint_url)
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let updated_permissions =
            self.props.current_user.resource_access != props.current_user.resource_access;
        self.props = props;
        if updated_permissions {
            self.link.send_message(Msg::GetPublishers);
        }
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
                        value=self.publisher_id
                        data=self.data.publishers.clone()
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                Msg::ChangePublisher(Uuid::parse_str(&value).unwrap_or_default())
                            }
                            _ => unreachable!(),
                        })
                        required = true
                    />
                    <FormTextInput
                        label = "Imprint Name"
                        value=self.imprint.imprint_name.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeImprintName(e.value))
                        required=true
                    />
                    <FormUrlInput
                        label = "Imprint URL"
                        value=self.imprint.imprint_url.clone()
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
