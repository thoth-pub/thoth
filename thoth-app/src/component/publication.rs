use thoth_api::account::model::AccountDetails;
use thoth_api::model::location::Location;
use thoth_api::model::price::Price;
use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationProperties;
use thoth_api::model::publication::PublicationWithRelations;
use thoth_api::model::work::WorkType;
use thoth_errors::ThothError;
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
use crate::component::delete_dialogue::ConfirmDeleteComponent;
use crate::component::locations_form::LocationsFormComponent;
use crate::component::prices_form::PricesFormComponent;
use crate::component::publication_modal::PublicationModalComponent;
use crate::component::utils::Loader;
use crate::models::publication::delete_publication_mutation::DeletePublicationRequest;
use crate::models::publication::delete_publication_mutation::DeletePublicationRequestBody;
use crate::models::publication::delete_publication_mutation::PushActionDeletePublication;
use crate::models::publication::delete_publication_mutation::PushDeletePublication;
use crate::models::publication::delete_publication_mutation::Variables as DeleteVariables;
use crate::models::publication::publication_query::FetchActionPublication;
use crate::models::publication::publication_query::FetchPublication;
use crate::models::publication::publication_query::PublicationRequest;
use crate::models::publication::publication_query::PublicationRequestBody;
use crate::models::publication::publication_query::Variables;
use crate::route::AdminRoute;
use crate::string::EDIT_BUTTON;
use crate::string::RELATIONS_INFO;

pub struct PublicationComponent {
    publication: PublicationWithRelations,
    fetch_publication: FetchPublication,
    delete_publication: PushDeletePublication,
    show_modal_form: bool,
    publication_under_edit: Option<Publication>,
    notification_bus: NotificationDispatcher,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleModalFormDisplay(bool),
    AddPublication(Publication),
    UpdatePublication(Publication),
    SetPublicationFetchState(FetchActionPublication),
    GetPublication,
    SetPublicationDeleteState(PushActionDeletePublication),
    DeletePublication,
    UpdateLocations(Option<Vec<Location>>),
    UpdatePrices(Option<Vec<Price>>),
}

#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub publication_id: Uuid,
    pub current_user: AccountDetails,
}

impl Component for PublicationComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let fetch_publication: FetchPublication = Default::default();
        let delete_publication = Default::default();
        let show_modal_form = false;
        let publication_under_edit = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let publication: PublicationWithRelations = Default::default();

        ctx.link().send_message(Msg::GetPublication);

        PublicationComponent {
            publication,
            fetch_publication,
            delete_publication,
            show_modal_form,
            publication_under_edit,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form) => {
                self.show_modal_form = show_form;
                // Opening the modal form from this form always means
                // we are about to edit the current publication
                self.publication_under_edit = match self.show_modal_form {
                    // Child form requires plain Publication, not PublicationWithRelations
                    true => Some(Publication {
                        publication_id: self.publication.publication_id,
                        publication_type: self.publication.publication_type.clone(),
                        work_id: self.publication.work_id,
                        isbn: self.publication.isbn.clone(),
                        // Not used by child form
                        created_at: Default::default(),
                        updated_at: self.publication.updated_at.clone(),
                        width_mm: self.publication.width_mm,
                        width_in: self.publication.width_in,
                        height_mm: self.publication.height_mm,
                        height_in: self.publication.height_in,
                        depth_mm: self.publication.depth_mm,
                        depth_in: self.publication.depth_in,
                        weight_g: self.publication.weight_g,
                        weight_oz: self.publication.weight_oz,
                    }),
                    false => None,
                };
                true
            }
            Msg::AddPublication(_p) => {
                // It should not be possible to call the child form from this component
                // in a way which creates a new publication (rather than editing an existing one).
                unreachable!()
            }
            Msg::UpdatePublication(p) => {
                if p.publication_id == self.publication.publication_id
                    && p.work_id == self.publication.work_id
                {
                    self.notification_bus.send(Request::NotificationBusMsg((
                        format!(
                            "Saved {}",
                            &p.isbn
                                .as_ref()
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| p.publication_id.to_string())
                        ),
                        NotificationStatus::Success,
                    )));
                    // Child form has updated the current publication - replace its values
                    // (need to convert from Publication back to PublicationWithRelations)
                    self.publication.publication_type = p.publication_type;
                    self.publication.isbn = p.isbn;
                    self.publication.updated_at = p.updated_at;
                    self.publication.width_mm = p.width_mm;
                    self.publication.width_in = p.width_in;
                    self.publication.height_mm = p.height_mm;
                    self.publication.height_in = p.height_in;
                    self.publication.depth_mm = p.depth_mm;
                    self.publication.depth_in = p.depth_in;
                    self.publication.weight_g = p.weight_g;
                    self.publication.weight_oz = p.weight_oz;
                } else {
                    // This should not be possible: the updated publication returned from the
                    // database does not match the locally-stored publication data.
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
            Msg::SetPublicationFetchState(fetch_state) => {
                self.fetch_publication.apply(fetch_state);
                match self.fetch_publication.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => {
                        self.publication = match &body.data.publication {
                            Some(c) => c.to_owned(),
                            None => Default::default(),
                        };
                        // If user doesn't have permission to edit this object, redirect to dashboard
                        if let Some(publishers) =
                            ctx.props().current_user.resource_access.restricted_to()
                        {
                            if !publishers.contains(
                                &self
                                    .publication
                                    .work
                                    .imprint
                                    .publisher
                                    .publisher_id
                                    .to_string(),
                            ) {
                                ctx.link().history().unwrap().push(AdminRoute::Dashboard);
                            }
                        }
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetPublication => {
                let body = PublicationRequestBody {
                    variables: Variables {
                        publication_id: Some(ctx.props().publication_id),
                    },
                    ..Default::default()
                };
                let request = PublicationRequest { body };
                self.fetch_publication = Fetch::new(request);

                ctx.link()
                    .send_future(self.fetch_publication.fetch(Msg::SetPublicationFetchState));
                ctx.link()
                    .send_message(Msg::SetPublicationFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetPublicationDeleteState(fetch_state) => {
                self.delete_publication.apply(fetch_state);
                match self.delete_publication.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_publication {
                        Some(p) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!(
                                    "Deleted {}",
                                    &p.isbn
                                        .as_ref()
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| p.publication_id.to_string())
                                ),
                                NotificationStatus::Success,
                            )));
                            ctx.link().history().unwrap().push(AdminRoute::Publications);
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
            Msg::DeletePublication => {
                let body = DeletePublicationRequestBody {
                    variables: DeleteVariables {
                        publication_id: self.publication.publication_id,
                    },
                    ..Default::default()
                };
                let request = DeletePublicationRequest { body };
                self.delete_publication = Fetch::new(request);
                ctx.link().send_future(
                    self.delete_publication
                        .fetch(Msg::SetPublicationDeleteState),
                );
                ctx.link()
                    .send_message(Msg::SetPublicationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::UpdateLocations(locations) => self.publication.locations.neq_assign(locations),
            Msg::UpdatePrices(prices) => self.publication.prices.neq_assign(prices),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.fetch_publication.as_ref().state() {
            FetchState::NotFetching(_) => html! {<Loader/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(_body) => {
                html! {
                    <>
                        <nav class="level">
                            <div class="level-left">
                                <p class="subtitle is-5">
                                    { "Edit publication" }
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
                                <PublicationModalComponent
                                    publication_under_edit={ self.publication_under_edit.clone() }
                                    work_id={ self.publication.work.work_id }
                                    work_type={ self.publication.work.work_type.clone() }
                                    show_modal_form={ self.show_modal_form }
                                    add_publication={ ctx.link().callback(Msg::AddPublication) }
                                    update_publication={ ctx.link().callback(Msg::UpdatePublication) }
                                    close_modal_form={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(false)) }
                                />
                                <p class="level-item">
                                    <ConfirmDeleteComponent
                                        onclick={ ctx.link().callback(|_| Msg::DeletePublication) }
                                        object_name={ self
                                            .publication.isbn
                                            .as_ref()
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| self.publication.publication_id.to_string())
                                            .clone()
                                        }
                                    />
                                </p>
                            </div>
                        </nav>

                        <form>
                            <div class="field">
                                <label class="label">{ "Publication Type" }</label>
                                <div class="control is-expanded">
                                    {&self.publication.publication_type}
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "ISBN" }</label>
                                <div class="control is-expanded">
                                    {&self.publication.isbn.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "".to_string())}
                                </div>
                            </div>
                        </form>

                        {
                            // Dimensions are only applicable to physical (Paperback/Hardback) non-Chapter publications.
                            if self.publication.is_physical() && self.publication.work.work_type != WorkType::BookChapter {
                                html! {
                                    <>
                                        <div class="field is-horizontal">
                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Width (mm)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.width_mm.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>

                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Height (mm)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.height_mm.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>

                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Depth (mm)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.depth_mm.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>

                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Weight (g)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.weight_g.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>
                                        </div>

                                        <div class="field is-horizontal">
                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Width (in)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.width_in.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>

                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Height (in)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.height_in.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>

                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Depth (in)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.depth_in.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>

                                            <div class="field" style="width: 8em;">
                                                <label class="label">{ "Weight (oz)" }</label>
                                                <div class="control is-expanded">
                                                    {&self.publication.weight_oz.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                                </div>
                                            </div>
                                        </div>
                                    </>
                                }
                            } else {
                                html!{}
                            }
                        }

                        <hr/>

                        <article class="message is-info">
                            <div class="message-body">
                                { RELATIONS_INFO }
                            </div>
                        </article>

                        <LocationsFormComponent
                            locations={ self.publication.locations.clone() }
                            publication_id={ self.publication.publication_id }
                            update_locations={ ctx.link().callback(Msg::UpdateLocations) }
                        />

                        <PricesFormComponent
                            prices={ self.publication.prices.clone() }
                            publication_id={ self.publication.publication_id }
                            update_prices={ ctx.link().callback(Msg::UpdatePrices) }
                        />
                    </>
                }
            }
            FetchState::Failed(_, err) => html! {
                { ThothError::from(err).to_string() }
            },
        }
    }
}
