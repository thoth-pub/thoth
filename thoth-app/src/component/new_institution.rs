use thoth_api::model::institution::Institution;
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
use crate::models::institution::create_institution_mutation::CreateInstitutionRequest;
use crate::models::institution::create_institution_mutation::CreateInstitutionRequestBody;
use crate::models::institution::create_institution_mutation::PushActionCreateInstitution;
use crate::models::institution::create_institution_mutation::PushCreateInstitution;
use crate::models::institution::create_institution_mutation::Variables;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct NewInstitutionComponent {
    institution: Institution,
    // Track the user-entered DOI string, which may not be validly formatted
    institution_doi: String,
    institution_doi_warning: String,
    push_institution: PushCreateInstitution,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetInstitutionPushState(PushActionCreateInstitution),
    CreateInstitution,
    ChangeInstitutionName(String),
    ChangeInstitutionDoi(String),
    ChangeRoute(AppRoute),
}

impl Component for NewInstitutionComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let push_institution = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let institution: Institution = Default::default();
        let institution_doi = Default::default();
        let institution_doi_warning = Default::default();
        let router = RouteAgentDispatcher::new();

        NewInstitutionComponent {
            institution,
            institution_doi,
            institution_doi_warning,
            push_institution,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetInstitutionPushState(fetch_state) => {
                self.push_institution.apply(fetch_state);
                match self.push_institution.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_institution {
                        Some(f) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", f.institution_name),
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
            Msg::CreateInstitution => {
                // Only update the DOI value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
                // If no DOI was provided, no format check is required.
                if self.institution_doi.is_empty() {
                    self.institution.institution_doi.neq_assign(None);
                } else if let Ok(result) = self.institution_doi.parse::<Doi>() {
                    self.institution.institution_doi.neq_assign(Some(result));
                }
                let body = CreateInstitutionRequestBody {
                    variables: Variables {
                        institution_name: self.institution.institution_name.clone(),
                        institution_doi: self.institution.institution_doi.clone(),
                    },
                    ..Default::default()
                };
                let request = CreateInstitutionRequest { body };
                self.push_institution = Fetch::new(request);
                self.link
                    .send_future(self.push_institution.fetch(Msg::SetInstitutionPushState));
                self.link
                    .send_message(Msg::SetInstitutionPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeInstitutionName(institution_name) => self
                .institution
                .institution_name
                .neq_assign(institution_name.trim().to_owned()),
            Msg::ChangeInstitutionDoi(value) => {
                if self.institution_doi.neq_assign(value.trim().to_owned()) {
                    // If DOI is not correctly formatted, display a warning.
                    // Don't update self.institution.institution_doi yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.institution_doi_warning.clear();
                    match self.institution_doi.parse::<Doi>() {
                        Err(e) => {
                            match e {
                                // If no DOI was provided, no warning is required.
                                ThothError::DoiEmptyError => {}
                                _ => self.institution_doi_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.institution_doi = value.to_string(),
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
            Msg::CreateInstitution
        });
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <p class="subtitle is-5">
                            { "New institution" }
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>

                <form onsubmit=callback>
                    <FormTextInput
                        label = "Institution Name"
                        value=self.institution.institution_name.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeInstitutionName(e.value))
                        required=true
                    />
                    <FormTextInputExtended
                        label = "Institution DOI"
                        statictext = DOI_DOMAIN
                        value=self.institution_doi.clone()
                        tooltip=self.institution_doi_warning.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ChangeInstitutionDoi(e.value))
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
