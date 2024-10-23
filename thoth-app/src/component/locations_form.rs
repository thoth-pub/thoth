use std::str::FromStr;
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
use crate::models::location::create_location_mutation::CreateLocationRequest;
use crate::models::location::create_location_mutation::CreateLocationRequestBody;
use crate::models::location::create_location_mutation::PushActionCreateLocation;
use crate::models::location::create_location_mutation::PushCreateLocation;
use crate::models::location::create_location_mutation::Variables as CreateVariables;
use crate::models::location::delete_location_mutation::DeleteLocationRequest;
use crate::models::location::delete_location_mutation::DeleteLocationRequestBody;
use crate::models::location::delete_location_mutation::PushActionDeleteLocation;
use crate::models::location::delete_location_mutation::PushDeleteLocation;
use crate::models::location::delete_location_mutation::Variables as DeleteVariables;
use crate::models::location::location_platforms_query::FetchActionLocationPlatforms;
use crate::models::location::location_platforms_query::FetchLocationPlatforms;
use crate::models::location::update_location_mutation::PushActionUpdateLocation;
use crate::models::location::update_location_mutation::PushUpdateLocation;
use crate::models::location::update_location_mutation::UpdateLocationRequest;
use crate::models::location::update_location_mutation::UpdateLocationRequestBody;
use crate::models::location::update_location_mutation::Variables as UpdateVariables;
use crate::models::location::LocationPlatformValues;
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_LOCATIONS;
use crate::string::NO;
use crate::string::REMOVE_BUTTON;
use crate::string::YES;

use super::ToElementValue;
use super::ToOption;

pub struct LocationsFormComponent {
    data: LocationsFormData,
    location: Location,
    show_modal_form: bool,
    in_edit_mode: bool,
    fetch_location_platforms: FetchLocationPlatforms,
    create_location: PushCreateLocation,
    delete_location: PushDeleteLocation,
    update_location: PushUpdateLocation,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct LocationsFormData {
    location_platforms: Vec<LocationPlatformValues>,
}

pub enum Msg {
    ToggleModalFormDisplay(bool, Option<Location>),
    SetLocationPlatformsFetchState(FetchActionLocationPlatforms),
    GetLocationPlatforms,
    SetLocationCreateState(PushActionCreateLocation),
    CreateLocation,
    SetLocationDeleteState(PushActionDeleteLocation),
    DeleteLocation(Uuid),
    SetLocationUpdateState(PushActionUpdateLocation),
    UpdateLocation,
    ChangeLandingPage(String),
    ChangeFullTextUrl(String),
    ChangeLocationPlatform(LocationPlatform),
    ChangeCanonical(bool),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub locations: Option<Vec<Location>>,
    pub publication_id: Uuid,
    pub update_locations: Callback<()>,
}

impl Component for LocationsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let data: LocationsFormData = Default::default();
        let show_modal_form = false;
        let in_edit_mode = false;
        // The first location needs to be canonical = true (as it will be
        // the only location); subsequent locations need to be canonical = false
        let location = Location {
            canonical: ctx.props().locations.as_ref().unwrap_or(&vec![]).is_empty(),
            ..Default::default()
        };
        let fetch_location_platforms = Default::default();
        let create_location = Default::default();
        let delete_location = Default::default();
        let update_location = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ctx.link().send_message(Msg::GetLocationPlatforms);

        LocationsFormComponent {
            data,
            location,
            show_modal_form,
            in_edit_mode,
            fetch_location_platforms,
            create_location,
            delete_location,
            update_location,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, l) => {
                self.show_modal_form = show_form;
                self.in_edit_mode = l.is_some();

                if self.in_edit_mode {
                    if let Some(location) = l {
                        // Editing existing location: load its current values.
                        self.location = location;
                    }
                } else {
                    self.location = Default::default();
                    self.location.canonical = true;
                    self.location.location_platform = LocationPlatform::Other;
                }
                true
            }
            Msg::SetLocationPlatformsFetchState(fetch_state) => {
                self.fetch_location_platforms.apply(fetch_state);
                self.data.location_platforms = match self.fetch_location_platforms.as_ref().state()
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
            Msg::SetLocationCreateState(fetch_state) => {
                self.create_location.apply(fetch_state);
                match self.create_location.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_location {
                        Some(l) => {
                            let location = l.clone();
                            let mut locations: Vec<Location> =
                                ctx.props().locations.clone().unwrap_or_default();
                            locations.push(location);
                            ctx.props().update_locations.emit(());
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            true
                        }
                        None => {
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link()
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateLocation => {
                let body = CreateLocationRequestBody {
                    variables: CreateVariables {
                        publication_id: ctx.props().publication_id,
                        landing_page: self.location.landing_page.clone(),
                        full_text_url: self.location.full_text_url.clone(),
                        location_platform: self.location.location_platform.clone(),
                        canonical: self.location.canonical,
                    },
                    ..Default::default()
                };
                let request = CreateLocationRequest { body };
                self.create_location = Fetch::new(request);
                ctx.link()
                    .send_future(self.create_location.fetch(Msg::SetLocationCreateState));
                ctx.link()
                    .send_message(Msg::SetLocationCreateState(FetchAction::Fetching));
                false
            }
            Msg::SetLocationUpdateState(fetch_state) => {
                self.update_location.apply(fetch_state);
                match self.update_location.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_location {
                        Some(_l) => {
                            ctx.props().update_locations.emit(());
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            // changed the return value to false below, but this doesn't fix the display
                            // issue where the page jumps during refresh when modal is exited
                            false
                        }
                        None => {
                            ctx.link()
                                .send_message(Msg::ToggleModalFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link()
                            .send_message(Msg::ToggleModalFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdateLocation => {
                let body = UpdateLocationRequestBody {
                    variables: UpdateVariables {
                        location_id: self.location.location_id,
                        publication_id: self.location.publication_id,
                        landing_page: self.location.landing_page.clone(),
                        full_text_url: self.location.full_text_url.clone(),
                        location_platform: self.location.location_platform.clone(),
                        canonical: self.location.canonical,
                    },
                    ..Default::default()
                };
                let request = UpdateLocationRequest { body };
                self.update_location = Fetch::new(request);
                ctx.link()
                    .send_future(self.update_location.fetch(Msg::SetLocationUpdateState));
                ctx.link()
                    .send_message(Msg::SetLocationUpdateState(FetchAction::Fetching));

                false
            }
            Msg::SetLocationDeleteState(fetch_state) => {
                self.delete_location.apply(fetch_state);
                match self.delete_location.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_location {
                        Some(_location) => {
                            ctx.props().update_locations.emit(());
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
            Msg::ChangeLandingPage(val) => {
                self.location.landing_page.neq_assign(val.to_opt_string())
            }
            Msg::ChangeFullTextUrl(val) => {
                self.location.full_text_url.neq_assign(val.to_opt_string())
            }
            Msg::ChangeLocationPlatform(code) => self.location.location_platform.neq_assign(code),
            Msg::ChangeCanonical(val) => self.location.canonical.neq_assign(val),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let locations = ctx.props().locations.clone().unwrap_or_default();
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(true, None)
        });
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(false, None)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Locations" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick={ open_modal }
                    >
                        { "Add Location" }
                    </button>
                </div>
                <div class={ self.modal_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ self.modal_form_title() }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="locations-form" onsubmit={ self.modal_form_action(ctx) }>
                                <FormUrlInput
                                    label="Landing Page"
                                    value={ self.location.landing_page.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeLandingPage(e.to_value())) }
                                />
                                <FormUrlInput
                                    label="Full Text URL"
                                    value={ self.location.full_text_url.clone() }
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
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="locations-form"
                            >
                                { self.modal_form_button() }
                            </button>
                            <button
                                class="button"
                                onclick={ &close_modal }
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !locations.is_empty() {
                        html!{{for locations.iter().map(|l| self.render_location(ctx, l))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_LOCATIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl LocationsFormComponent {
    fn modal_form_status(&self) -> String {
        match self.show_modal_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn modal_form_title(&self) -> String {
        match self.in_edit_mode {
            true => "Edit Location".to_string(),
            false => "New Location".to_string(),
        }
    }

    fn modal_form_button(&self) -> String {
        match self.in_edit_mode {
            true => "Save Location".to_string(),
            false => "Add Location".to_string(),
        }
    }

    fn modal_form_action(&self, ctx: &Context<Self>) -> Callback<FocusEvent> {
        match self.in_edit_mode {
            true => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdateLocation
            }),
            false => ctx.link().callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::CreateLocation
            }),
        }
    }

    fn render_location(&self, ctx: &Context<Self>, l: &Location) -> Html {
        let location = l.clone();
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

                    <div class="field is-grouped is-grouped-right">
                        <div class="control">
                            <a
                                class="button is-success"
                                onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true, Some(location.clone()))) }
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
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
