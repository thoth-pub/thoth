use thoth_api::model::publisher::Publisher;
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
use crate::component::utils::FormUrlInput;
use crate::models::publisher::create_publisher_mutation::CreatePublisherRequest;
use crate::models::publisher::create_publisher_mutation::CreatePublisherRequestBody;
use crate::models::publisher::create_publisher_mutation::PushActionCreatePublisher;
use crate::models::publisher::create_publisher_mutation::PushCreatePublisher;
use crate::models::publisher::create_publisher_mutation::Variables;
use crate::models::EditRoute;
use crate::string::SAVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct NewPublisherComponent {
    publisher: Publisher,
    push_publisher: PushCreatePublisher,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetPublisherPushState(PushActionCreatePublisher),
    CreatePublisher,
    ChangePublisherName(String),
    ChangePublisherShortname(String),
    ChangePublisherUrl(String),
}

impl Component for NewPublisherComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let push_publisher = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let publisher: Publisher = Default::default();

        NewPublisherComponent {
            publisher,
            push_publisher,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                            ctx.link().history().unwrap().push(p.edit_route());
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
                ctx.link()
                    .send_future(self.push_publisher.fetch(Msg::SetPublisherPushState));
                ctx.link()
                    .send_message(Msg::SetPublisherPushState(FetchAction::Fetching));
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
        let callback = ctx.link().callback(|event: FocusEvent| {
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
}
