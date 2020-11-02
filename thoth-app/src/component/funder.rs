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
use crate::component::utils::Loader;
use crate::models::funder::funder_query::FetchActionFunder;
use crate::models::funder::funder_query::FetchFunder;
use crate::models::funder::funder_query::FunderRequest;
use crate::models::funder::funder_query::FunderRequestBody;
use crate::models::funder::funder_query::Variables;
use crate::models::funder::update_funder_mutation::PushActionUpdateFunder;
use crate::models::funder::update_funder_mutation::PushUpdateFunder;
use crate::models::funder::update_funder_mutation::UpdateFunderRequest;
use crate::models::funder::update_funder_mutation::UpdateFunderRequestBody;
use crate::models::funder::update_funder_mutation::Variables as UpdateVariables;
use crate::models::funder::Funder;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct FunderComponent {
    funder: Funder,
    fetch_funder: FetchFunder,
    push_funder: PushUpdateFunder,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetFunderFetchState(FetchActionFunder),
    GetFunder,
    SetFunderPushState(PushActionUpdateFunder),
    UpdateFunder,
    ChangeFunderName(String),
    ChangeFunderDoi(String),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub funder_id: String,
}

impl Component for FunderComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let body = FunderRequestBody {
            variables: Variables {
                funder_id: Some(props.funder_id),
            },
            ..Default::default()
        };
        let request = FunderRequest { body };
        let fetch_funder = Fetch::new(request);
        let push_funder = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let funder: Funder = Default::default();
        let router = RouteAgentDispatcher::new();

        link.send_message(Msg::GetFunder);

        FunderComponent {
            funder,
            fetch_funder,
            push_funder,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFunderFetchState(fetch_state) => {
                self.fetch_funder.apply(fetch_state);
                match self.fetch_funder.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        self.funder = match &body.data.funder {
                            Some(c) => c.to_owned(),
                            None => Default::default(),
                        };
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetFunder => {
                self.link
                    .send_future(self.fetch_funder.fetch(Msg::SetFunderFetchState));
                self.link
                    .send_message(Msg::SetFunderFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetFunderPushState(fetch_state) => {
                self.push_funder.apply(fetch_state);
                match self.push_funder.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_funder {
                        Some(f) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", f.funder_name),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(AdminRoute::Funders)));
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
            Msg::UpdateFunder => {
                let body = UpdateFunderRequestBody {
                    variables: UpdateVariables {
                        funder_id: self.funder.funder_id.clone(),
                        funder_name: self.funder.funder_name.clone(),
                        funder_doi: self.funder.funder_doi.clone(),
                    },
                    ..Default::default()
                };
                let request = UpdateFunderRequest { body };
                self.push_funder = Fetch::new(request);
                self.link
                    .send_future(self.push_funder.fetch(Msg::SetFunderPushState));
                self.link
                    .send_message(Msg::SetFunderPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeFunderName(funder_name) => self.funder.funder_name.neq_assign(funder_name),
            Msg::ChangeFunderDoi(funder_doi) => self.funder.funder_doi.neq_assign(Some(funder_doi)),
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
        match self.fetch_funder.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = self.link.callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::UpdateFunder
                });
                html! {
                    <form onsubmit=callback>
                        <FormTextInput
                            label = "Funder Name"
                            value=&self.funder.funder_name
                            oninput=self.link.callback(|e: InputData| Msg::ChangeFunderName(e.value))
                            required=true
                        />
                        <FormUrlInput
                            label = "Funder DOI"
                            value=&self.funder.funder_doi
                            oninput=self.link.callback(|e: InputData| Msg::ChangeFunderDoi(e.value))
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
