use thoth_api::funder::model::Funder;
use thoth_api::model::{Doi, DOI_DOMAIN};
use thoth_errors::ThothError;
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
use crate::component::utils::FormTextInputExtended;
use crate::models::funder::create_funder_mutation::CreateFunderRequest;
use crate::models::funder::create_funder_mutation::CreateFunderRequestBody;
use crate::models::funder::create_funder_mutation::PushActionCreateFunder;
use crate::models::funder::create_funder_mutation::PushCreateFunder;
use crate::models::funder::create_funder_mutation::Variables;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewFunderComponent {
    funder: Funder,
    // Track the user-entered DOI string, which may not be validly formatted
    funder_doi: String,
    funder_doi_warning: String,
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
        let funder_doi = Default::default();
        let funder_doi_warning = Default::default();
        let router = RouteAgentDispatcher::new();

        NewFunderComponent {
            funder,
            funder_doi,
            funder_doi_warning,
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
                            self.link.send_message(Msg::ChangeRoute(f.edit_route()));
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
                // Only update the DOI value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
                // If no DOI was provided, no format check is required.
                if self.funder_doi.is_empty() {
                    self.funder.funder_doi.neq_assign(None);
                } else if let Ok(result) = self.funder_doi.parse::<Doi>() {
                    self.funder.funder_doi.neq_assign(Some(result));
                }
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
            Msg::ChangeFunderName(funder_name) => self
                .funder
                .funder_name
                .neq_assign(funder_name.trim().to_owned()),
            Msg::ChangeFunderDoi(value) => {
                if self.funder_doi.neq_assign(value.trim().to_owned()) {
                    // If DOI is not correctly formatted, display a warning.
                    // Don't update self.funder.funder_doi yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.funder_doi_warning.clear();
                    match self.funder_doi.parse::<Doi>() {
                        Err(e) => {
                            match e {
                                // If no DOI was provided, no warning is required.
                                ThothError::DoiEmptyError => {}
                                _ => self.funder_doi_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.funder_doi = value.to_string(),
                    }
                    true
                } else {
                    false
                }
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
                        value=self.funder.funder_name.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeFunderName(e.value))
                        required=true
                    />
                    <FormTextInputExtended
                        label = "Funder DOI"
                        statictext = DOI_DOMAIN
                        value=self.funder_doi.clone()
                        tooltip=self.funder_doi_warning.clone()
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
