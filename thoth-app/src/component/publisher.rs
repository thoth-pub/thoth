use thoth_api::account::model::AccountDetails;
use thoth_api::model::publisher::Publisher;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yew_router::history::History;
use yew_router::prelude::RouterScopeExt;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::delete_dialogue::ConfirmDeleteComponent;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::component::utils::Loader;
use crate::models::publisher::delete_publisher_mutation::DeletePublisherRequest;
use crate::models::publisher::delete_publisher_mutation::DeletePublisherRequestBody;
use crate::models::publisher::delete_publisher_mutation::PushActionDeletePublisher;
use crate::models::publisher::delete_publisher_mutation::PushDeletePublisher;
use crate::models::publisher::delete_publisher_mutation::Variables as DeleteVariables;
use crate::models::publisher::publisher_query::FetchActionPublisher;
use crate::models::publisher::publisher_query::FetchPublisher;
use crate::models::publisher::publisher_query::PublisherRequest;
use crate::models::publisher::publisher_query::PublisherRequestBody;
use crate::models::publisher::publisher_query::Variables;
use crate::models::publisher::update_publisher_mutation::PushActionUpdatePublisher;
use crate::models::publisher::update_publisher_mutation::PushUpdatePublisher;
use crate::models::publisher::update_publisher_mutation::UpdatePublisherRequest;
use crate::models::publisher::update_publisher_mutation::UpdatePublisherRequestBody;
use crate::models::publisher::update_publisher_mutation::Variables as UpdateVariables;
use crate::route::AdminRoute;
use crate::string::SAVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct PublisherComponent {
    publisher: Publisher,
    fetch_publisher: FetchPublisher,
    push_publisher: PushUpdatePublisher,
    delete_publisher: PushDeletePublisher,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetPublisherFetchState(FetchActionPublisher),
    GetPublisher,
    SetPublisherPushState(PushActionUpdatePublisher),
    UpdatePublisher,
    SetPublisherDeleteState(PushActionDeletePublisher),
    DeletePublisher,
    ChangePublisherName(String),
    ChangePublisherShortname(String),
    ChangePublisherUrl(String),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub publisher_id: Uuid,
    pub current_user: AccountDetails,
}

impl Component for PublisherComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let fetch_publisher: FetchPublisher = Default::default();
        let push_publisher = Default::default();
        let delete_publisher = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let publisher: Publisher = Default::default();

        ctx.link().send_message(Msg::GetPublisher);

        PublisherComponent {
            publisher,
            fetch_publisher,
            push_publisher,
            delete_publisher,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetPublisherFetchState(fetch_state) => {
                self.fetch_publisher.apply(fetch_state);
                match self.fetch_publisher.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        self.publisher = match &body.data.publisher {
                            Some(c) => c.to_owned(),
                            None => Default::default(),
                        };
                        // If user doesn't have permission to edit this object, redirect to dashboard
                        if let Some(publishers) =
                            ctx.props().current_user.resource_access.restricted_to()
                        {
                            if !publishers.contains(&self.publisher.publisher_id.to_string()) {
                                ctx.link().history().unwrap().push(AdminRoute::Dashboard);
                            }
                        }
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetPublisher => {
                let body = PublisherRequestBody {
                    variables: Variables {
                        publisher_id: Some(ctx.props().publisher_id),
                    },
                    ..Default::default()
                };
                let request = PublisherRequest { body };
                self.fetch_publisher = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_publisher.fetch(Msg::SetPublisherFetchState));
                ctx.link()
                    .send_message(Msg::SetPublisherFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetPublisherPushState(fetch_state) => {
                self.push_publisher.apply(fetch_state);
                match self.push_publisher.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_publisher {
                        Some(p) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", p.publisher_name),
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
            Msg::UpdatePublisher => {
                let body = UpdatePublisherRequestBody {
                    variables: UpdateVariables {
                        publisher_id: self.publisher.publisher_id,
                        publisher_name: self.publisher.publisher_name.clone(),
                        publisher_shortname: self.publisher.publisher_shortname.clone(),
                        publisher_url: self.publisher.publisher_url.clone(),
                    },
                    ..Default::default()
                };
                let request = UpdatePublisherRequest { body };
                self.push_publisher = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_publisher.fetch(Msg::SetPublisherPushState));
                ctx.link()
                    .send_message(Msg::SetPublisherPushState(FetchAction::Fetching));
                false
            }
            Msg::SetPublisherDeleteState(fetch_state) => {
                self.delete_publisher.apply(fetch_state);
                match self.delete_publisher.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_publisher {
                        Some(f) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Deleted {}", f.publisher_name),
                                NotificationStatus::Success,
                            )));
                            ctx.link().history().unwrap().push(AdminRoute::Publishers);
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
            Msg::DeletePublisher => {
                let body = DeletePublisherRequestBody {
                    variables: DeleteVariables {
                        publisher_id: self.publisher.publisher_id,
                    },
                    ..Default::default()
                };
                let request = DeletePublisherRequest { body };
                self.delete_publisher = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_publisher.fetch(Msg::SetPublisherDeleteState));
                ctx.link()
                    .send_message(Msg::SetPublisherDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublisherName(publisher_name) => self
                .publisher
                .publisher_name
                .neq_assign(publisher_name.trim().to_owned()),
            Msg::ChangePublisherShortname(value) => self
                .publisher
                .publisher_shortname
                .neq_assign(value.to_opt_string()),
            Msg::ChangePublisherUrl(value) => self
                .publisher
                .publisher_url
                .neq_assign(value.to_opt_string()),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.fetch_publisher.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = ctx.link().callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::UpdatePublisher
                });
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit publisher" }
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                    <ConfirmDeleteComponent
                                        onclick={ ctx.link().callback(|_| Msg::DeletePublisher) }
                                        object_name={ self.publisher.publisher_name.clone() }
                                    />
                                </p>
                            </div>
                        </nav>

                        <form onsubmit={ callback }>
                            <FormTextInput
                                label = "Publisher Name"
                                value={ self.publisher.publisher_name.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePublisherName(e.to_value())) }
                                required = true
                            />
                            <FormTextInput
                                label = "Publisher Short Name"
                                value={ self.publisher.publisher_shortname.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePublisherShortname(e.to_value())) }
                            />
                            <FormUrlInput
                                label = "Publisher URL"
                                value={ self.publisher.publisher_url.clone() }
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangePublisherUrl(e.to_value())) }
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
