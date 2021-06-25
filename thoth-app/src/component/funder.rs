use thoth_api::funder::model::Funder;
use thoth_api::funding::model::FundingWithWork;
use thoth_api::model::{Doi, DOI_DOMAIN};
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::prelude::RouterAnchor;
use yew_router::route::Route;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::funder_activity_checker::FunderActivityChecker;
use crate::agent::funder_activity_checker::Request as FunderActivityRequest;
use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::delete_dialogue::ConfirmDeleteComponent;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormTextInputExtended;
use crate::component::utils::Loader;
use crate::models::funder::delete_funder_mutation::DeleteFunderRequest;
use crate::models::funder::delete_funder_mutation::DeleteFunderRequestBody;
use crate::models::funder::delete_funder_mutation::PushActionDeleteFunder;
use crate::models::funder::delete_funder_mutation::PushDeleteFunder;
use crate::models::funder::delete_funder_mutation::Variables as DeleteVariables;
use crate::models::funder::funder_activity_query::FunderActivityResponseData;
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
use crate::models::EditRoute;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::SAVE_BUTTON;

pub struct FunderComponent {
    funder: Funder,
    // Track the user-entered DOI string, which may not be validly formatted
    funder_doi: String,
    funder_doi_warning: String,
    fetch_funder: FetchFunder,
    push_funder: PushUpdateFunder,
    delete_funder: PushDeleteFunder,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
    _funder_activity_checker: Box<dyn Bridge<FunderActivityChecker>>,
    funder_activity: Vec<FundingWithWork>,
}

pub enum Msg {
    GetFunderActivity(FunderActivityResponseData),
    SetFunderFetchState(FetchActionFunder),
    GetFunder,
    SetFunderPushState(PushActionUpdateFunder),
    UpdateFunder,
    SetFunderDeleteState(PushActionDeleteFunder),
    DeleteFunder,
    ChangeFunderName(String),
    ChangeFunderDoi(String),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub funder_id: Uuid,
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
        let delete_funder = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let funder: Funder = Default::default();
        let funder_doi = Default::default();
        let funder_doi_warning = Default::default();
        let router = RouteAgentDispatcher::new();
        let mut _funder_activity_checker =
            FunderActivityChecker::bridge(link.callback(Msg::GetFunderActivity));
        let funder_activity = Default::default();

        link.send_message(Msg::GetFunder);
        _funder_activity_checker.send(FunderActivityRequest::RetrieveFunderActivity(
            props.funder_id,
        ));

        FunderComponent {
            funder,
            funder_doi,
            funder_doi_warning,
            fetch_funder,
            push_funder,
            delete_funder,
            link,
            router,
            notification_bus,
            _funder_activity_checker,
            funder_activity,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetFunderActivity(response) => {
                let mut should_render = false;
                if let Some(funder) = response.funder {
                    if let Some(fundings) = funder.fundings {
                        if !fundings.is_empty() {
                            self.funder_activity = fundings;
                            should_render = true;
                        }
                    }
                }
                should_render
            }
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
                        // Initialise user-entered DOI variable to match DOI in database
                        self.funder_doi = self
                            .funder
                            .funder_doi
                            .clone()
                            .unwrap_or_default()
                            .to_string();
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
                            // Save was successful: update user-entered DOI variable to match DOI in database
                            self.funder_doi = self
                                .funder
                                .funder_doi
                                .clone()
                                .unwrap_or_default()
                                .to_string();
                            self.funder_doi_warning.clear();
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Saved {}", f.funder_name),
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
            Msg::UpdateFunder => {
                // Only update the DOI value with the current user-entered string
                // if it is validly formatted - otherwise keep the database version.
                // If no DOI was provided, no format check is required.
                if self.funder_doi.is_empty() {
                    self.funder.funder_doi.neq_assign(None);
                } else if let Ok(result) = self.funder_doi.parse::<Doi>() {
                    self.funder.funder_doi.neq_assign(Some(result));
                }
                let body = UpdateFunderRequestBody {
                    variables: UpdateVariables {
                        funder_id: self.funder.funder_id,
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
            Msg::SetFunderDeleteState(fetch_state) => {
                self.delete_funder.apply(fetch_state);
                match self.delete_funder.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_funder {
                        Some(f) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Deleted {}", f.funder_name),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Funders,
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
            Msg::DeleteFunder => {
                let body = DeleteFunderRequestBody {
                    variables: DeleteVariables {
                        funder_id: self.funder.funder_id,
                    },
                    ..Default::default()
                };
                let request = DeleteFunderRequest { body };
                self.delete_funder = Fetch::new(request);
                self.link
                    .send_future(self.delete_funder.fetch(Msg::SetFunderDeleteState));
                self.link
                    .send_message(Msg::SetFunderDeleteState(FetchAction::Fetching));
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
        match self.fetch_funder.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                let callback = self.link.callback(|event: FocusEvent| {
                    event.prevent_default();
                    Msg::UpdateFunder
                });
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit funder" }
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                    <ConfirmDeleteComponent
                                        onclick=self.link.callback(|_| Msg::DeleteFunder)
                                        object_name=self.funder.funder_name.clone()
                                    />
                                </p>
                            </div>
                        </nav>

                        { if !self.funder_activity.is_empty() {
                            html! {
                                <div class="notification is-link">
                                    {
                                        for self.funder_activity.iter().map(|funding| {
                                            html! {
                                                <p>
                                                    { "Funded: " }
                                                    <RouterAnchor<AppRoute>
                                                        route=funding.work.edit_route()
                                                    >
                                                        { &funding.work.title }
                                                    </  RouterAnchor<AppRoute>>
                                                    { format!(", from: {}", funding.work.imprint.publisher.publisher_name) }
                                                </p>
                                            }
                                        })
                                    }
                                </div>
                                }
                            } else {
                                html! {}
                            }
                        }

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
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
