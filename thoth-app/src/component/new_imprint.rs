use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::imprint::Imprint;
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
use crate::string::SAVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct NewImprintComponent {
    imprint: Imprint,
    publisher_id: Uuid,
    push_imprint: PushCreateImprint,
    data: ImprintFormData,
    fetch_publishers: FetchPublishers,
    notification_bus: NotificationDispatcher,
    // Store props value locally in order to test whether it has been updated on props change
    resource_access: AccountAccess,
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
}
#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub current_user: AccountDetails,
}

impl Component for NewImprintComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let push_imprint = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let imprint: Imprint = Default::default();
        let publisher_id: Uuid = Default::default();
        let data: ImprintFormData = Default::default();
        let fetch_publishers: FetchPublishers = Default::default();
        let resource_access = ctx.props().current_user.resource_access.clone();

        ctx.link().send_message(Msg::GetPublishers);

        NewImprintComponent {
            imprint,
            publisher_id,
            push_imprint,
            data,
            fetch_publishers,
            notification_bus,
            resource_access,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
                        publishers: ctx.props().current_user.resource_access.restricted_to(),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = PublishersRequest { body };
                self.fetch_publishers = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_publishers.fetch(Msg::SetPublishersFetchState));
                ctx.link()
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
                            ctx.link().history().unwrap().push(i.edit_route());
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
                ctx.link()
                    .send_future(self.push_imprint.fetch(Msg::SetImprintPushState));
                ctx.link()
                    .send_message(Msg::SetImprintPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublisher(publisher_id) => self.publisher_id.neq_assign(publisher_id),
            Msg::ChangeImprintName(imprint_name) => self
                .imprint
                .imprint_name
                .neq_assign(imprint_name.trim().to_owned()),
            Msg::ChangeImprintUrl(value) => {
                self.imprint.imprint_url.neq_assign(value.to_opt_string())
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access.clone());
        if updated_permissions {
            ctx.link().send_message(Msg::GetPublishers);
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(|event: FocusEvent| {
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

                <form onsubmit={ callback }>
                    <FormPublisherSelect
                        label = "Publisher"
                        value={ self.publisher_id }
                        data={ self.data.publishers.clone() }
                        onchange={ ctx.link().callback(|e: Event|
                            Msg::ChangePublisher(Uuid::parse_str(&e.to_value()).unwrap_or_default())
                        ) }
                        required = true
                    />
                    <FormTextInput
                        label = "Imprint Name"
                        value={ self.imprint.imprint_name.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeImprintName(e.to_value())) }
                        required = true
                    />
                    <FormUrlInput
                        label = "Imprint URL"
                        value={ self.imprint.imprint_url.clone() }
                        oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeImprintUrl(e.to_value())) }
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
