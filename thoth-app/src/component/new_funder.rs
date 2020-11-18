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
use crate::models::funder::create_funder_mutation::CreateFunderRequest;
use crate::models::funder::create_funder_mutation::CreateFunderRequestBody;
use crate::models::funder::create_funder_mutation::PushActionCreateFunder;
use crate::models::funder::create_funder_mutation::PushCreateFunder;
use crate::models::funder::create_funder_mutation::Variables;
use crate::models::funder::Funder;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewFunderComponent {
    funder: Funder,
    push_funder: PushCreateFunder,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetFunderPushState(PushActionCreateFunder),
    CreateFunder,
    ChangeFunderName(String),
    ChangeFunderDoi(String),
    ChangeRoute(AppRoute),
}

impl Component for NewFunderComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_funder = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let funder: Funder = Default::default();
        let router = RouteAgentDispatcher::new();

        NewFunderComponent {
            funder,
            push_funder,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFunderPushState(fetch_state) => {
                self.push_funder.apply(fetch_state);
                match self.push_funder.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_funder {
                        Some(f) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", f.funder_name),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Funder(f.funder_id.clone()),
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
            Msg::CreateFunder => {
                let body = CreateFunderRequestBody {
                    variables: Variables {
                        funder_name: self.funder.funder_name.clone(),
                        funder_doi: self.funder.funder_doi.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateFunderRequest { body };
                self.push_funder = Fetch::new(request);
                self.link
                    .send_future(self.push_funder.fetch(Msg::SetFunderPushState));
                self.link
                    .send_message(Msg::SetFunderPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeFunderName(funder_name) => {
                self.funder.funder_name.neq_assign(funder_name.trim().to_owned())
            }
            Msg::ChangeFunderDoi(value) => {
                let funder_doi = match value.trim().is_empty() {
                    true => None,
                    false => Some(value.trim().to_owned()),
                };
                self.funder.funder_doi.neq_assign(funder_doi)
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
            Msg::CreateFunder
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New funder" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

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
            </>
        }
    }
}
