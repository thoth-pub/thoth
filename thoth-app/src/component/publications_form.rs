use std::str::FromStr;
use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationProperties;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::work::WorkType;
use thoth_api::model::{Convert, Isbn, LengthUnit, WeightUnit};
use thoth_errors::ThothError;
use uuid::Uuid;
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
use crate::component::utils::FormFloatInput;
use crate::component::utils::FormPublicationTypeSelect;
use crate::component::utils::FormTextInputExtended;
use crate::models::publication::create_publication_mutation::CreatePublicationRequest;
use crate::models::publication::create_publication_mutation::CreatePublicationRequestBody;
use crate::models::publication::create_publication_mutation::PushActionCreatePublication;
use crate::models::publication::create_publication_mutation::PushCreatePublication;
use crate::models::publication::create_publication_mutation::Variables;
use crate::models::publication::delete_publication_mutation::DeletePublicationRequest;
use crate::models::publication::delete_publication_mutation::DeletePublicationRequestBody;
use crate::models::publication::delete_publication_mutation::PushActionDeletePublication;
use crate::models::publication::delete_publication_mutation::PushDeletePublication;
use crate::models::publication::delete_publication_mutation::Variables as DeleteVariables;
use crate::models::publication::publication_types_query::FetchActionPublicationTypes;
use crate::models::publication::publication_types_query::FetchPublicationTypes;
use crate::models::publication::update_publication_mutation::PushActionUpdatePublication;
use crate::models::publication::update_publication_mutation::PushUpdatePublication;
use crate::models::publication::update_publication_mutation::UpdatePublicationRequest;
use crate::models::publication::update_publication_mutation::UpdatePublicationRequestBody;
use crate::models::publication::update_publication_mutation::Variables as UpdateVariables;
use crate::models::publication::PublicationTypeValues;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::CANCEL_BUTTON;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_PUBLICATIONS;
use crate::string::REMOVE_BUTTON;
use crate::string::VIEW_BUTTON;

use super::ToOption;

pub struct PublicationsFormComponent {
    props: Props,
    data: PublicationsFormData,
    new_publication: Publication,
    // Track the user-entered ISBN string, which may not be validly formatted
    isbn: String,
    isbn_warning: String,
    show_add_form: bool,
    convert_dimensions: bool,
    fetch_publication_types: FetchPublicationTypes,
    push_publication: PushCreatePublication,
    delete_publication: PushDeletePublication,
    update_publication: PushUpdatePublication,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
}

#[derive(Default)]
struct PublicationsFormData {
    publication_types: Vec<PublicationTypeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool, Option<Publication>),
    ToggleDimensionConversion,
    SetPublicationTypesFetchState(FetchActionPublicationTypes),
    GetPublicationTypes,
    SetPublicationPushState(PushActionCreatePublication),
    CreatePublication,
    SetPublicationUpdateState(PushActionUpdatePublication),
    UpdatePublication,
    SetPublicationDeleteState(PushActionDeletePublication),
    DeletePublication(Uuid),
    ChangePublicationType(PublicationType),
    ChangeIsbn(String),
    ChangeWidthMm(String),
    ChangeWidthIn(String),
    ChangeHeightMm(String),
    ChangeHeightIn(String),
    ChangeDepthMm(String),
    ChangeDepthIn(String),
    ChangeWeightG(String),
    ChangeWeightOz(String),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub publications: Option<Vec<Publication>>,
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub update_publications: Callback<Option<Vec<Publication>>>,
}

impl Component for PublicationsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: PublicationsFormData = Default::default();
        let show_add_form = false;
        let convert_dimensions = true;
        let new_publication: Publication = Default::default();
        let isbn = Default::default();
        let isbn_warning = Default::default();
        let push_publication = Default::default();
        let delete_publication = Default::default();
        let update_publication = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let router = RouteAgentDispatcher::new();

        link.send_message(Msg::GetPublicationTypes);

        PublicationsFormComponent {
            props,
            data,
            new_publication,
            isbn,
            isbn_warning,
            show_add_form,
            convert_dimensions,
            fetch_publication_types: Default::default(),
            push_publication,
            delete_publication,
            update_publication,
            link,
            notification_bus,
            router,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value, p) => {
                self.show_add_form = value;
                if value {
                    if let Some(publication) = p {
                        // Editing existing publication: load its current values.
                        self.new_publication = publication;
                    } else {
                        // Creating new publication: clear any previous values.
                        self.new_publication = Default::default();
                    }
                    // Ensure ISBN variable value is kept in sync with publication object.
                    self.isbn = self
                        .new_publication
                        .isbn()
                        .clone()
                        .unwrap_or_default()
                        .to_string();
                    // Clear ISBN warning as the variable value is now valid by definition
                    // (it has either been set to a database value or the default empty value)
                    self.isbn_warning = Default::default();
                }
                true
            }
            Msg::ToggleDimensionConversion => {
                self.convert_dimensions = !self.convert_dimensions;
                false
            }
            Msg::SetPublicationTypesFetchState(fetch_state) => {
                self.fetch_publication_types.apply(fetch_state);
                self.data.publication_types = match self.fetch_publication_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.publication_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetPublicationTypes => {
                self.link.send_future(
                    self.fetch_publication_types
                        .fetch(Msg::SetPublicationTypesFetchState),
                );
                self.link
                    .send_message(Msg::SetPublicationTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetPublicationPushState(fetch_state) => {
                self.push_publication.apply(fetch_state);
                match self.push_publication.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_publication {
                        Some(p) => {
                            let publication = p.clone();
                            let mut publications: Vec<Publication> =
                                self.props.publications.clone().unwrap_or_default();
                            publications.push(publication);
                            self.props.update_publications.emit(Some(publications));
                            self.link
                                .send_message(Msg::ToggleAddFormDisplay(false, None));
                            true
                        }
                        None => {
                            self.link
                                .send_message(Msg::ToggleAddFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link
                            .send_message(Msg::ToggleAddFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreatePublication => {
                // Only update the ISBN value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
                // If no ISBN was provided, no format check is required.
                if self.isbn.is_empty() {
                    self.new_publication.isbn.neq_assign(None);
                } else if let Ok(result) = self.isbn.parse::<Isbn>() {
                    self.new_publication.isbn.neq_assign(Some(result));
                }
                // Clear any fields which are not applicable to the currently selected work/publication type.
                // (Do not clear them before the save point as the user may change the type again.)
                if self.new_publication.is_digital()
                    || self.props.work_type == WorkType::BookChapter
                {
                    self.new_publication.width_mm = None;
                    self.new_publication.width_in = None;
                    self.new_publication.height_mm = None;
                    self.new_publication.height_in = None;
                    self.new_publication.depth_mm = None;
                    self.new_publication.depth_in = None;
                    self.new_publication.weight_g = None;
                    self.new_publication.weight_oz = None;
                }
                let body = CreatePublicationRequestBody {
                    variables: Variables {
                        work_id: self.props.work_id,
                        publication_type: self.new_publication.publication_type.clone(),
                        isbn: self.new_publication.isbn.clone(),
                        width_mm: self.new_publication.width_mm,
                        width_in: self.new_publication.width_in,
                        height_mm: self.new_publication.height_mm,
                        height_in: self.new_publication.height_in,
                        depth_mm: self.new_publication.depth_mm,
                        depth_in: self.new_publication.depth_in,
                        weight_g: self.new_publication.weight_g,
                        weight_oz: self.new_publication.weight_oz,
                    },
                    ..Default::default()
                };
                let request = CreatePublicationRequest { body };
                self.push_publication = Fetch::new(request);
                self.link
                    .send_future(self.push_publication.fetch(Msg::SetPublicationPushState));
                self.link
                    .send_message(Msg::SetPublicationPushState(FetchAction::Fetching));
                false
            }
            Msg::SetPublicationUpdateState(fetch_state) => {
                self.update_publication.apply(fetch_state);
                match self.update_publication.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.update_publication {
                        Some(p) => {
                            let mut publications: Vec<Publication> =
                                self.props.publications.clone().unwrap_or_default();
                            if let Some(publication) = publications
                                .iter_mut()
                                .find(|pb| pb.publication_id == p.publication_id)
                            {
                                *publication = p.clone();
                            }
                            self.props.update_publications.emit(Some(publications));
                            self.link
                                .send_message(Msg::ToggleAddFormDisplay(false, None));
                            true
                        }
                        None => {
                            self.link
                                .send_message(Msg::ToggleAddFormDisplay(false, None));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link
                            .send_message(Msg::ToggleAddFormDisplay(false, None));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::UpdatePublication => {
                // Only update the ISBN value with the current user-entered string
                // if it is validly formatted - otherwise keep the default.
                // If no ISBN was provided, no format check is required.
                if self.isbn.is_empty() {
                    self.new_publication.isbn.neq_assign(None);
                } else if let Ok(result) = self.isbn.parse::<Isbn>() {
                    self.new_publication.isbn.neq_assign(Some(result));
                }
                // Clear any fields which are not applicable to the currently selected work/publication type.
                // (Do not clear them before the save point as the user may change the type again.)
                if self.new_publication.is_digital()
                    || self.props.work_type == WorkType::BookChapter
                {
                    self.new_publication.width_mm = None;
                    self.new_publication.width_in = None;
                    self.new_publication.height_mm = None;
                    self.new_publication.height_in = None;
                    self.new_publication.depth_mm = None;
                    self.new_publication.depth_in = None;
                    self.new_publication.weight_g = None;
                    self.new_publication.weight_oz = None;
                }
                let body = UpdatePublicationRequestBody {
                    variables: UpdateVariables {
                        publication_id: self.new_publication.publication_id,
                        work_id: self.props.work_id,
                        publication_type: self.new_publication.publication_type.clone(),
                        isbn: self.new_publication.isbn.clone(),
                        width_mm: self.new_publication.width_mm,
                        width_in: self.new_publication.width_in,
                        height_mm: self.new_publication.height_mm,
                        height_in: self.new_publication.height_in,
                        depth_mm: self.new_publication.depth_mm,
                        depth_in: self.new_publication.depth_in,
                        weight_g: self.new_publication.weight_g,
                        weight_oz: self.new_publication.weight_oz,
                    },
                    ..Default::default()
                };
                let request = UpdatePublicationRequest { body };
                self.update_publication = Fetch::new(request);
                self.link.send_future(
                    self.update_publication
                        .fetch(Msg::SetPublicationUpdateState),
                );
                self.link
                    .send_message(Msg::SetPublicationUpdateState(FetchAction::Fetching));
                false
            }
            Msg::SetPublicationDeleteState(fetch_state) => {
                self.delete_publication.apply(fetch_state);
                match self.delete_publication.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_publication {
                        Some(publication) => {
                            let to_keep: Vec<Publication> = self
                                .props
                                .publications
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|p| p.publication_id != publication.publication_id)
                                .collect();
                            self.props.update_publications.emit(Some(to_keep));
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
            Msg::DeletePublication(publication_id) => {
                let body = DeletePublicationRequestBody {
                    variables: DeleteVariables { publication_id },
                    ..Default::default()
                };
                let request = DeletePublicationRequest { body };
                self.delete_publication = Fetch::new(request);
                self.link.send_future(
                    self.delete_publication
                        .fetch(Msg::SetPublicationDeleteState),
                );
                self.link
                    .send_message(Msg::SetPublicationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublicationType(val) => {
                self.new_publication.publication_type.neq_assign(val)
            }
            Msg::ChangeIsbn(value) => {
                if self.isbn.neq_assign(value.trim().to_owned()) {
                    // If ISBN is not correctly formatted, display a warning.
                    // Don't update self.new_publication.isbn yet, as user may later
                    // overwrite a new valid value with an invalid one.
                    self.isbn_warning.clear();
                    match self.isbn.parse::<Isbn>() {
                        Err(e) => {
                            match e {
                                // If no ISBN was provided, no warning is required.
                                ThothError::IsbnEmptyError => {}
                                _ => self.isbn_warning = e.to_string(),
                            }
                        }
                        Ok(value) => self.isbn = value.to_string(),
                    }
                    true
                } else {
                    false
                }
            }
            Msg::ChangeWidthMm(value) => {
                let changed_value = self
                    .new_publication
                    .width_mm
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut width_in = None;
                    // Automatically update paired length field with default conversion.
                    if let Some(width_mm) = self.new_publication.width_mm {
                        width_in =
                            Some(width_mm.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::In));
                    }
                    self.new_publication.width_in.neq_assign(width_in);
                }
                changed_value
            }
            Msg::ChangeWidthIn(value) => {
                let changed_value = self
                    .new_publication
                    .width_in
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut width_mm = None;
                    // Automatically update paired length field with default conversion.
                    if let Some(width_in) = self.new_publication.width_in {
                        width_mm =
                            Some(width_in.convert_length_from_to(&LengthUnit::In, &LengthUnit::Mm));
                    }
                    self.new_publication.width_mm.neq_assign(width_mm);
                }
                changed_value
            }
            Msg::ChangeHeightMm(value) => {
                let changed_value = self
                    .new_publication
                    .height_mm
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut height_in = None;
                    // Automatically update paired length field with default conversion.
                    if let Some(height_mm) = self.new_publication.height_mm {
                        height_in = Some(
                            height_mm.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::In),
                        );
                    }
                    self.new_publication.height_in.neq_assign(height_in);
                }
                changed_value
            }
            Msg::ChangeHeightIn(value) => {
                let changed_value = self
                    .new_publication
                    .height_in
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut height_mm = None;
                    // Automatically update paired length field with default conversion.
                    if let Some(height_in) = self.new_publication.height_in {
                        height_mm = Some(
                            height_in.convert_length_from_to(&LengthUnit::In, &LengthUnit::Mm),
                        );
                    }
                    self.new_publication.height_mm.neq_assign(height_mm);
                }
                changed_value
            }
            Msg::ChangeDepthMm(value) => {
                let changed_value = self
                    .new_publication
                    .depth_mm
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut depth_in = None;
                    // Automatically update paired length field with default conversion.
                    if let Some(depth_mm) = self.new_publication.depth_mm {
                        depth_in =
                            Some(depth_mm.convert_length_from_to(&LengthUnit::Mm, &LengthUnit::In));
                    }
                    self.new_publication.depth_in.neq_assign(depth_in);
                }
                changed_value
            }
            Msg::ChangeDepthIn(value) => {
                let changed_value = self
                    .new_publication
                    .depth_in
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut depth_mm = None;
                    // Automatically update paired length field with default conversion.
                    if let Some(depth_in) = self.new_publication.depth_in {
                        depth_mm =
                            Some(depth_in.convert_length_from_to(&LengthUnit::In, &LengthUnit::Mm));
                    }
                    self.new_publication.depth_mm.neq_assign(depth_mm);
                }
                changed_value
            }
            Msg::ChangeWeightG(value) => {
                let changed_value = self
                    .new_publication
                    .weight_g
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut weight_oz = None;
                    // Automatically update paired weight field with default conversion.
                    if let Some(weight_g) = self.new_publication.weight_g {
                        weight_oz =
                            Some(weight_g.convert_weight_from_to(&WeightUnit::G, &WeightUnit::Oz));
                    }
                    self.new_publication.weight_oz.neq_assign(weight_oz);
                }
                changed_value
            }
            Msg::ChangeWeightOz(value) => {
                let changed_value = self
                    .new_publication
                    .weight_oz
                    .neq_assign(value.to_opt_float());
                if changed_value && self.convert_dimensions {
                    let mut weight_g = None;
                    // Automatically update paired weight field with default conversion.
                    if let Some(weight_oz) = self.new_publication.weight_oz {
                        weight_g =
                            Some(weight_oz.convert_weight_from_to(&WeightUnit::Oz, &WeightUnit::G));
                    }
                    self.new_publication.weight_g.neq_assign(weight_g);
                }
                changed_value
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let publications = self.props.publications.clone().unwrap_or_default();
        let open_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(true, None)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false, None)
        });
        let mut form_title = "New Publication";
        let mut add_or_save_publication = "Add Publication";
        let mut submit_publication = self.link.callback(|e: FocusEvent| {
            e.prevent_default();
            Msg::CreatePublication
        });
        if self.new_publication.publication_id != Uuid::default() {
            // If the ID is set to the default, we're creating a new publication
            // If the ID is anything else, we're editing an existing publication
            form_title = "Edit Publication";
            add_or_save_publication = "Save Publication";
            submit_publication = self.link.callback(|e: FocusEvent| {
                e.prevent_default();
                Msg::UpdatePublication
            });
        }
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Publications" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick=open_modal
                    >
                        { "Add Publication" }
                    </button>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ form_title }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="publications-form" onsubmit=submit_publication>
                                <FormPublicationTypeSelect
                                    label = "Publication Type"
                                    value=self.new_publication.publication_type.clone()
                                    data=self.data.publication_types.clone()
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangePublicationType(
                                                PublicationType::from_str(&value).unwrap()
                                            )
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormTextInputExtended
                                    label = "ISBN"
                                    value=self.isbn.clone()
                                    tooltip=self.isbn_warning.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeIsbn(e.value))
                                    // ISBNs cannot be added for publications whose work type is Book Chapter.
                                    deactivated=self.props.work_type == WorkType::BookChapter
                                />
                                {
                                    // Dimensions can only be added for physical (Paperback/Hardback) non-Chapter publications.
                                    if self.new_publication.is_physical() && self.props.work_type != WorkType::BookChapter {
                                        html! {
                                            <>
                                                <label class="checkbox">
                                                    <input
                                                        type="checkbox"
                                                        checked=self.convert_dimensions
                                                        onchange=self.link.callback(|event| match event {
                                                            ChangeData::Value(_) => Msg::ToggleDimensionConversion,
                                                            _ => unreachable!(),
                                                        })
                                                    />
                                                    { "Automatically convert dimension values" }
                                                </label>
                                                <div class="field is-horizontal">
                                                    <div class="field-body">
                                                        <FormFloatInput
                                                            label = "Width (mm)"
                                                            value=self.new_publication.width_mm
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeWidthMm(e.value))
                                                            step="1".to_string()
                                                        />
                                                        <FormFloatInput
                                                            label = "Width (in)"
                                                            value=self.new_publication.width_in
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeWidthIn(e.value))
                                                            step="0.01".to_string()
                                                        />
                                                    </div>
                                                </div>
                                                <div class="field is-horizontal">
                                                    <div class="field-body">
                                                        <FormFloatInput
                                                            label = "Height (mm)"
                                                            value=self.new_publication.height_mm
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeHeightMm(e.value))
                                                            step="1".to_string()
                                                        />
                                                        <FormFloatInput
                                                            label = "Height (in)"
                                                            value=self.new_publication.height_in
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeHeightIn(e.value))
                                                            step="0.01".to_string()
                                                        />
                                                    </div>
                                                </div>
                                                <div class="field is-horizontal">
                                                    <div class="field-body">
                                                        <FormFloatInput
                                                            label = "Depth (mm)"
                                                            value=self.new_publication.depth_mm
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeDepthMm(e.value))
                                                            step="1".to_string()
                                                        />
                                                        <FormFloatInput
                                                            label = "Depth (in)"
                                                            value=self.new_publication.depth_in
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeDepthIn(e.value))
                                                            step="0.01".to_string()
                                                        />
                                                    </div>
                                                </div>
                                                <div class="field is-horizontal">
                                                    <div class="field-body">
                                                        <FormFloatInput
                                                            label = "Weight (g)"
                                                            value=self.new_publication.weight_g
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeWeightG(e.value))
                                                            step="1".to_string()
                                                        />
                                                        <FormFloatInput
                                                            label = "Weight (oz)"
                                                            value=self.new_publication.weight_oz
                                                            oninput=self.link.callback(|e: InputData| Msg::ChangeWeightOz(e.value))
                                                            step="0.0001".to_string()
                                                        />
                                                    </div>
                                                </div>
                                            </>
                                        }
                                    } else {
                                        html!{}
                                    }
                                }
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="publications-form"
                            >
                                { add_or_save_publication }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !publications.is_empty() {
                        html!{{for publications.iter().map(|p| self.render_publication(p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_PUBLICATIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl PublicationsFormComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_publication(&self, p: &Publication) -> Html {
        let publication = p.clone();
        let publication_id = p.publication_id;
        let route = p.edit_route();
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-atlas" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Publication Type" }</label>
                        <div class="control is-expanded">
                            {&p.publication_type}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "ISBN" }</label>
                        <div class="control is-expanded">
                            {&p.isbn.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>

                    {
                        // Dimensions are only applicable to physical (Paperback/Hardback) non-Chapter publications.
                        if p.is_physical() && self.props.work_type != WorkType::BookChapter {
                            html! {
                                <>
                                    <div class="field is-vertical">
                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Width (mm)" }</label>
                                            <div class="control is-expanded">
                                                {&p.width_mm.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>

                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Width (in)" }</label>
                                            <div class="control is-expanded">
                                                {&p.width_in.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>
                                    </div>

                                    <div class="field is-vertical">
                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Height (mm)" }</label>
                                            <div class="control is-expanded">
                                                {&p.height_mm.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>

                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Height (in)" }</label>
                                            <div class="control is-expanded">
                                                {&p.height_in.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>
                                    </div>

                                    <div class="field is-vertical">
                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Depth (mm)" }</label>
                                            <div class="control is-expanded">
                                                {&p.depth_mm.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>

                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Depth (in)" }</label>
                                            <div class="control is-expanded">
                                                {&p.depth_in.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>
                                    </div>

                                    <div class="field is-vertical">
                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Weight (g)" }</label>
                                            <div class="control is-expanded">
                                                {&p.weight_g.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>

                                        <div class="field" style="width: 8em;">
                                            <label class="label">{ "Weight (oz)" }</label>
                                            <div class="control is-expanded">
                                                {&p.weight_oz.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                                            </div>
                                        </div>
                                    </div>
                                </>
                            }
                        } else {
                            html!{}
                        }
                    }

                    <div class="field is-grouped is-grouped-right">
                        <div class="control">
                            <a
                                class="button is-success"
                                onclick=self.link.callback(move |_| Msg::ToggleAddFormDisplay(true, Some(publication.clone())))
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-info"
                                onclick=self.link.callback(move |_| Msg::ChangeRoute(route.clone()))
                            >
                                { VIEW_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::DeletePublication(publication_id))
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
