use std::str::FromStr;
use thoth_api::account::model::AccountDetails;
use thoth_api::model::location::Location;
use thoth_api::model::location::LocationPlatform;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormBooleanSelect;
use crate::component::utils::FormLocationPlatformSelect;
use crate::component::utils::FormUrlInput;
use crate::component::utils::Loader;
use crate::models::location::delete_location_mutation::DeleteLocationRequest;
use crate::models::location::delete_location_mutation::DeleteLocationRequestBody;
use crate::models::location::delete_location_mutation::PushActionDeleteLocation;
use crate::models::location::delete_location_mutation::PushDeleteLocation;
use crate::models::location::delete_location_mutation::Variables as DeleteVariables;
use crate::models::location::location_query::FetchActionLocation;
use crate::models::location::location_query::FetchLocation;
use crate::models::location::location_query::LocationRequest;
use crate::models::location::location_query::LocationRequestBody;
use crate::models::location::location_query::Variables;
use crate::models::location::location_platforms_query::FetchActionLocationPlatforms;
use crate::models::location::location_platforms_query::FetchLocationPlatforms;
use crate::models::location::LocationPlatformValues;
use crate::route::AdminRoute;
use crate::string::EDIT_BUTTON;
use crate::string::YES;
use crate::string::NO;
use crate::string::REMOVE_BUTTON;

use super::ToElementValue;
use super::ToOption;

pub struct LocationComponent {
    location: Location,
    fetch_location: FetchLocation,
    delete_location: PushDeleteLocation,
    show_modal_form: bool,
    location_under_edit: Option<Location>,
    fetch_location_platforms: FetchLocationPlatforms,
    notification_bus: NotificationDispatcher,
}
pub struct LocationsFormComponent {
    data: LocationsFormData,
    new_location: Location,
    show_add_form: bool,
    fetch_location_platforms: FetchLocationPlatforms,
    delete_location: PushDeleteLocation,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct LocationsFormData {
    location_platforms: Vec<LocationPlatformValues>,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleModalFormDisplay(bool),
    UpdateLocation(Location),
    SetLocationFetchState(FetchActionLocation),
    SetLocationPlatformsFetchState(FetchActionLocationPlatforms),
    GetLocation,
    GetLocationPlatforms,
    SetLocationDeleteState(PushActionDeleteLocation),
    DeleteLocation(Uuid),
    ChangeLandingPage(String),
    ChangeFullTextUrl(String),
    ChangeLocationPlatform(LocationPlatform),
    ChangeCanonical(bool),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub locations: Option<Vec<Location>>,
    pub publication_id: Uuid,
    pub update_locations: Callback<Option<Vec<Location>>>,
    pub current_user: AccountDetails,
}

impl Component for LocationComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let fetch_location: FetchLocation = Default::default();
        let delete_location = Default::default();
        let show_modal_form = false;
        let fetch_location_platforms = Default::default();
        let location_under_edit = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let location: Location = Default::default();

        ctx.link().send_message(Msg::GetLocation);
        ctx.link().send_message(Msg::GetLocationPlatforms);

        LocationComponent {
            location,
            fetch_location,
            delete_location,
            show_modal_form,
            location_under_edit,
            fetch_location_platforms,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let mut data: LocationsFormData = Default::default();
        match msg {
            Msg::ToggleModalFormDisplay(show_form) => {
                self.show_modal_form = show_form;
                // Opening the modal form from this form always means
                // we are about to edit the current location
                self.location_under_edit = match self.show_modal_form {
                    true => Some(Location {
                        location_id: self.location.location_id,
                        publication_id: self.location.publication_id,
                        created_at: Default::default(),
                        updated_at: self.location.updated_at.clone(),
                        landing_page: self.location.landing_page.clone(),
                        full_text_url: self.location.full_text_url.clone(),
                        location_platform: self.location.location_platform.clone(),
                        canonical: self.location.canonical,
                    }),
                    false => None,
                };
                true
            }
            Msg::UpdateLocation(l) => {
                if l.location_id == self.location.location_id
                    && l.publication_id == self.location.publication_id
                {
                    self.notification_bus.send(Request::NotificationBusMsg((
                        format!(
                            "Saved {}",
                            &l.location_id
                                .to_string()
                        ),
                        NotificationStatus::Success,
                    )));
                    // Child form has updated the current location - replace its values
                    self.location.updated_at = l.updated_at;
                    self.location.landing_page = l.landing_page;
                    self.location.full_text_url = l.full_text_url;
                    self.location.location_platform = l.location_platform;
                    self.location.canonical = l.canonical;
                } else {
                    // This should not be possible: the updated location returned from the
                    // database does not match the locally-stored location data.
                    // Refreshing the page will reload the local data from the database.
                    self.notification_bus.send(Request::NotificationBusMsg((
                        "Changes were saved but display failed to update. Refresh your browser to view current data.".to_string(),
                        NotificationStatus::Warning,
                    )));
                }
                // Close child form
                ctx.link().send_message(Msg::ToggleModalFormDisplay(false));
                true
            }
            Msg::SetLocationFetchState(fetch_state) => {
                self.fetch_location.apply(fetch_state);
                match self.fetch_location.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        // self.location = match &body.data.location {
                        //     Some(c) => c.to_owned(),
                        //     None => Default::default(),
                        // };
                        // If user doesn't have permission to edit this object, redirect to dashboard
                        // TODO: implement this. But how can I get to this from 
                        // location? I don't have publisher_id, just publication_id
                        // if let Some(publishers) =
                        //     ctx.props().current_user.resource_access.restricted_to()
                        // {
                        //     if !publishers.contains(
                        //         &self
                        //             .publication
                        //             .work
                        //             .imprint
                        //             .publisher
                        //             .publisher_id
                        //             .to_string(),
                        //     ) {
                        //         ctx.link().history().unwrap().push(AdminRoute::Dashboard);
                        //     }
                        // }
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetLocation => {
                let body = LocationRequestBody {
                    variables: Variables {
                        location_id: Some(ctx.props().publication_id),
                    },
                    ..Default::default()
                };
                let request = LocationRequest { body };
                self.fetch_location = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_location.fetch(Msg::SetLocationFetchState));
                ctx.link()
                    .send_message(Msg::SetLocationFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetLocationPlatformsFetchState(fetch_state) => {
                self.fetch_location_platforms.apply(fetch_state);
                data.location_platforms = match self.fetch_location_platforms.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.location_platforms.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetLocationPlatforms => {
                ctx.link().send_future(
                    self.fetch_location_platforms
                        .fetch(Msg::SetLocationPlatformsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetLocationPlatformsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetLocationDeleteState(fetch_state) => {
                self.delete_location.apply(fetch_state);
                match self.delete_location.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_location {
                        Some(location) => {
                            let to_keep: Vec<Location> = ctx
                                .props()
                                .locations
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|l| l.location_id != location.location_id)
                                .collect();
                            ctx.props().update_locations.emit(Some(to_keep));
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
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::DeleteLocation(location_id) => {
                let body = DeleteLocationRequestBody {
                    variables: DeleteVariables { location_id },
                    ..Default::default()
                };
                let request = DeleteLocationRequest { body };
                self.delete_location = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_location.fetch(Msg::SetLocationDeleteState));
                ctx.link()
                    .send_message(Msg::SetLocationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeLandingPage(val) => self
                .location
                .landing_page
                .neq_assign(val.to_opt_string()),
            Msg::ChangeFullTextUrl(val) => self
                .location
                .full_text_url
                .neq_assign(val.to_opt_string()),
            Msg::ChangeLocationPlatform(code) => {
                self.location.location_platform.neq_assign(code)
            }
            Msg::ChangeCanonical(val) => self.location.canonical.neq_assign(val),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let locations = ctx.props().locations.clone().unwrap_or_default();
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(true)
        });
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false)
        });
        match self.fetch_location.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit location" }
                                </p>
                            </div>
                            <div class="level-right">
                                <p class="level-item">
                                    <a
                                        class="button is-success"
                                        onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true)) }
                                    >
                                        { EDIT_BUTTON }
                                    </a>
                                </p>
                            </div>
                        </nav>
                                <div class={ self.open_modal() }>
                                    <div class="modal-background" onclick={ &close_modal }></div>
                                    <div class="modal-card">
                                        <header class="modal-card-head">
                                            <p class="modal-card-title">{ "Edit Location" }</p>
                                            <button
                                                class="delete"
                                                aria-label="close"
                                                onclick={ &close_modal }
                                            ></button>
                                        </header>
                                        <section class="modal-card-body">
                                            <form id="locations-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                                e.prevent_default();
                                                Msg::EditLocation
                                            }) }
                                            >
                                                <FormUrlInput
                                                    label="Landing Page"
                                                    value={ self.location.landing_page.clone() }
                                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLandingPage(e.to_value())) }
                                                />
                                                <FormUrlInput
                                                    label="Full Text URL"
                                                    value={ self.location.full_text_url.clone().unwrap_or_default() }
                                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeFullTextUrl(e.to_value())) }
                                                />
                                                <FormLocationPlatformSelect
                                                    label = "Location Platform"
                                                    value={ self.location.location_platform.clone() }
                                                    data={ self.data.location_platforms.clone() }
                                                    onchange={ ctx.link().callback(|e: Event|
                                                        Msg::ChangeLocationPlatform(LocationPlatform::from_str(&e.to_value()).unwrap())
                                                    ) }
                                                    required = true
                                                />
                                                <FormBooleanSelect
                                                    label = "Canonical"
                                                    value={ self.location.canonical }
                                                    onchange={ ctx.link().callback(|e: Event|
                                                        Msg::ChangeCanonical(e.to_value() == "true")
                                                    ) }
                                                    required = true
                                                />
                                            </form>
                                        </section>
                                    </div>
                                </div>
                    </>
                }
            }
            FetchState::Failed(_, err) => html! {
                { ThothError::from(err).to_string() }
            },
        }
    }
}

impl LocationsFormComponent {
    fn edit_form_status(&self) -> String {
        match self.show_edit_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_location(&self, ctx: &Context<Self>, l: &Location) -> Html {
        let location_id = l.location_id;
        let mut delete_callback = Some(
            ctx.link()
                .callback(move |_| Msg::DeleteLocation(location_id)),
        );
        let mut delete_deactivated = false;
        // If the location is canonical and other (non-canonical) locations exist, prevent it from
        // being deleted by deactivating the delete button and unsetting its callback attribute
        if l.canonical && ctx.props().locations.as_ref().unwrap_or(&vec![]).len() > 1 {
            delete_callback = None;
            delete_deactivated = true;
        }
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-file-invoice-dollar" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 11em; word-wrap: break-word;">
                        <label class="label">{ "Landing Page" }</label>
                        <div class="control is-expanded">
                            {&l.landing_page.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field" style="width: 11em; word-wrap: break-word;">
                        <label class="label">{ "Full Text URL" }</label>
                        <div class="control is-expanded">
                            {&l.full_text_url.clone().unwrap_or_default()}
                        </div>
                    </div>
                    <div class="field" style="width: 5em;">
                        <label class="label">{ "Platform" }</label>
                        <div class="control is-expanded">
                            {&l.location_platform}
                        </div>
                    </div>
                    <div class="field" style="width: 5em;">
                        <label class="label">{ "Canonical" }</label>
                        <div class="control is-expanded">
                            {
                                match l.canonical {
                                    true => { YES },
                                    false => { NO }
                                }
                            }
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick={ delete_callback }
                                disabled={ delete_deactivated }
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
