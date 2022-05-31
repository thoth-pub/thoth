use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::imprint::ImprintWithPublisher;
use thoth_api::model::publisher::Publisher;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
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
use crate::component::delete_dialogue::ConfirmDeleteComponent;
use crate::component::utils::FormPublisherSelect;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::component::utils::Loader;
use crate::models::imprint::delete_imprint_mutation::DeleteImprintRequest;
use crate::models::imprint::delete_imprint_mutation::DeleteImprintRequestBody;
use crate::models::imprint::delete_imprint_mutation::PushActionDeleteImprint;
use crate::models::imprint::delete_imprint_mutation::PushDeleteImprint;
use crate::models::imprint::delete_imprint_mutation::Variables as DeleteVariables;
use crate::models::imprint::imprint_query::FetchActionImprint;
use crate::models::imprint::imprint_query::FetchImprint;
use crate::models::imprint::imprint_query::ImprintRequest;
use crate::models::imprint::imprint_query::ImprintRequestBody;
use crate::models::imprint::imprint_query::Variables;
use crate::models::imprint::update_imprint_mutation::PushActionUpdateImprint;
use crate::models::imprint::update_imprint_mutation::PushUpdateImprint;
use crate::models::imprint::update_imprint_mutation::UpdateImprintRequest;
use crate::models::imprint::update_imprint_mutation::UpdateImprintRequestBody;
use crate::models::imprint::update_imprint_mutation::Variables as UpdateVariables;
use crate::models::publisher::publishers_query::FetchActionPublishers;
use crate::models::publisher::publishers_query::FetchPublishers;
use crate::models::publisher::publishers_query::PublishersRequest;
use crate::models::publisher::publishers_query::PublishersRequestBody;
use crate::models::publisher::publishers_query::Variables as PublishersVariables;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

use super::ToOption;

pub struct ImprintComponent {
    imprint: ImprintWithPublisher,
    fetch_imprint: FetchImprint,
    push_imprint: PushUpdateImprint,
    delete_imprint: PushDeleteImprint,
    data: ImprintFormData,
    fetch_publishers: FetchPublishers,
    router: RouteAgentDispatcher<()>,
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
    SetImprintFetchState(FetchActionImprint),
    GetImprint,
    SetImprintPushState(PushActionUpdateImprint),
    UpdateImprint,
    SetImprintDeleteState(PushActionDeleteImprint),
    DeleteImprint,
    ChangePublisher(Uuid),
    ChangeImprintName(String),
    ChangeImprintUrl(String),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub imprint_id: Uuid,
    pub current_user: AccountDetails,
}

impl Component for ImprintComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let fetch_imprint: FetchImprint = Default::default();
        let data: ImprintFormData = Default::default();
        let fetch_publishers: FetchPublishers = Default::default();
        let push_imprint = Default::default();
        let delete_imprint = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let imprint: ImprintWithPublisher = Default::default();
        let router = RouteAgentDispatcher::new();
        let resource_access = ctx.props().current_user.resource_access;

        ctx.link().send_message(Msg::GetImprint);
        ctx.link().send_message(Msg::GetPublishers);

        ImprintComponent {
            imprint,
            fetch_imprint,
            push_imprint,
            delete_imprint,
            data,
            fetch_publishers,
            router,
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
                        publishers: self.resource_access.restricted_to(),
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
            Msg::SetImprintFetchState(fetch_state) => {
                self.fetch_imprint.apply(fetch_state);
                match self.fetch_imprint.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        self.imprint = match &body.data.imprint {
                            Some(c) => c.to_owned(),
                            None => Default::default(),
                        };
                        // If user doesn't have permission to edit this object, redirect to dashboard
                        if let Some(publishers) = self.resource_access.restricted_to() {
                            if !publishers
                                .contains(&self.imprint.publisher.publisher_id.to_string())
                            {
                                self.router.send(RouteRequest::ChangeRoute(Route::from(
                                    AppRoute::Admin(AdminRoute::Dashboard),
                                )));
                            }
                        }
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetImprint => {
                let body = ImprintRequestBody {
                    variables: Variables {
                        imprint_id: Some(ctx.props().imprint_id),
                    },
                    ..Default::default()
                };
                let request = ImprintRequest { body };
                self.fetch_imprint = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_imprint.fetch(Msg::SetImprintFetchState));
                ctx.link()
                    .send_message(Msg::SetImprintFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetImprintPushState(fetch_state) => {
                self.push_imprint.apply(fetch_state);
                match self.push_imprint.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_imprint {
                        Some(i) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", i.imprint_name),
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
            Msg::UpdateImprint => {
                let body = UpdateImprintRequestBody {
                    variables: UpdateVariables {
                        imprint_id: self.imprint.imprint_id,
                        imprint_name: self.imprint.imprint_name.clone(),
                        imprint_url: self.imprint.imprint_url.clone(),
                        publisher_id: self.imprint.publisher.publisher_id,
                    },
                    ..Default::default()
                };
                let request = UpdateImprintRequest { body };
                self.push_imprint = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_imprint.fetch(Msg::SetImprintPushState));
                ctx.link()
                    .send_message(Msg::SetImprintPushState(FetchAction::Fetching));
                false
            }
            Msg::SetImprintDeleteState(fetch_state) => {
                self.delete_imprint.apply(fetch_state);
                match self.delete_imprint.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_imprint {
                        Some(i) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Deleted {}", i.imprint_name),
                                NotificationStatus::Success,
                            )));
                            ctx.link().send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Imprints,
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
            Msg::DeleteImprint => {
                let body = DeleteImprintRequestBody {
                    variables: DeleteVariables {
                        imprint_id: self.imprint.imprint_id,
                    },
                    ..Default::default()
                };
                let request = DeleteImprintRequest { body };
                self.delete_imprint = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_imprint.fetch(Msg::SetImprintDeleteState));
                ctx.link()
                    .send_message(Msg::SetImprintDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublisher(publisher_id) => {
                if let Some(publisher) = self
                    .data
                    .publishers
                    .iter()
                    .find(|p| p.publisher_id == publisher_id)
                {
                    self.imprint.publisher.neq_assign(publisher.clone())
                } else {
                    // Publisher not found: clear existing selection
                    self.imprint.publisher.neq_assign(Default::default())
                }
            }
            Msg::ChangeImprintName(imprint_name) => self
                .imprint
                .imprint_name
                .neq_assign(imprint_name.trim().to_owned()),
            Msg::ChangeImprintUrl(value) => {
                self.imprint.imprint_url.neq_assign(value.to_opt_string())
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access);
        if updated_permissions {
            ctx.link().send_message(Msg::GetPublishers);
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.fetch_imprint.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = ctx.link().callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::UpdateImprint
                });
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit imprint" }
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                    <ConfirmDeleteComponent
                                        onclick={ ctx.link().callback(|_| Msg::DeleteImprint) }
                                        object_name={ self.imprint.imprint_name.clone() }
                                    />
                                </p>
                            </div>
                        </nav>

                        <form onsubmit={ callback }>
                            <FormPublisherSelect
                                label = "Publisher"
                                value={ self.imprint.publisher.publisher_id }
                                data={ self.data.publishers.clone() }
                                onchange={ ctx.link().callback(|event| match event {
                                    ChangeData::Select(elem) => {
                                        let value = elem.value();
                                        Msg::ChangePublisher(Uuid::parse_str(&value).unwrap_or_default())
                                    }
                                    _ => unreachable!(),
                                }) }
                                required = true
                            />
                            <FormTextInput
                                label = "Imprint Name"
                                value={ self.imprint.imprint_name.clone() }
                                oninput={ ctx.link().callback(|e: InputData| Msg::ChangeImprintName(e.value)) }
                                required = true
                            />
                            <FormUrlInput
                                label = "Imprint URL"
                                value={ self.imprint.imprint_url.clone() }
                                oninput={ ctx.link().callback(|e: InputData| Msg::ChangeImprintUrl(e.value)) }
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
