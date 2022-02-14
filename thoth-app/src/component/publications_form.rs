use std::str::FromStr;
use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationType;
use thoth_api::model::work::WorkType;
use thoth_api::model::Isbn;
use thoth_api::model::WeightUnit;
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
use crate::component::utils::FormWeightUnitSelect;
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
use crate::models::publication::weight_units_query::FetchActionWeightUnits;
use crate::models::publication::weight_units_query::FetchWeightUnits;
use crate::models::publication::PublicationTypeValues;
use crate::models::publication::WeightUnitValues;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::CANCEL_BUTTON;
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
    fetch_publication_types: FetchPublicationTypes,
    fetch_weight_units: FetchWeightUnits,
    push_publication: PushCreatePublication,
    delete_publication: PushDeletePublication,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
}

#[derive(Default)]
struct PublicationsFormData {
    publication_types: Vec<PublicationTypeValues>,
    weight_units: Vec<WeightUnitValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetPublicationTypesFetchState(FetchActionPublicationTypes),
    GetPublicationTypes,
    SetWeightUnitsFetchState(FetchActionWeightUnits),
    GetWeightUnits,
    SetPublicationPushState(PushActionCreatePublication),
    CreatePublication,
    SetPublicationDeleteState(PushActionDeletePublication),
    DeletePublication(Uuid),
    ChangePublicationType(PublicationType),
    ChangeIsbn(String),
    ChangeWeight(String),
    ChangeWeightUnit(WeightUnit),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub publications: Option<Vec<Publication>>,
    pub work_id: Uuid,
    pub work_type: WorkType,
    pub update_publications: Callback<Option<Vec<Publication>>>,
    pub weight_units_selection: WeightUnit,
    pub update_weight_units_selection: Callback<WeightUnit>,
}

impl Component for PublicationsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: PublicationsFormData = Default::default();
        let show_add_form = false;
        let new_publication: Publication = Default::default();
        let isbn = Default::default();
        let isbn_warning = Default::default();
        let push_publication = Default::default();
        let delete_publication = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let router = RouteAgentDispatcher::new();

        link.send_message(Msg::GetPublicationTypes);
        link.send_message(Msg::GetWeightUnits);

        PublicationsFormComponent {
            props,
            data,
            new_publication,
            isbn,
            isbn_warning,
            show_add_form,
            fetch_publication_types: Default::default(),
            fetch_weight_units: Default::default(),
            push_publication,
            delete_publication,
            link,
            notification_bus,
            router,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                // Ensure ISBN variables are cleared on re-opening form,
                // otherwise a previously-entered valid ISBN value may be
                // saved although an invalid value was subsequently entered
                self.new_publication.isbn = Default::default();
                self.isbn = Default::default();
                self.isbn_warning = Default::default();
                true
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
            Msg::SetWeightUnitsFetchState(fetch_state) => {
                self.fetch_weight_units.apply(fetch_state);
                self.data.weight_units = match self.fetch_weight_units.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.weight_units.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetWeightUnits => {
                self.link
                    .send_future(self.fetch_weight_units.fetch(Msg::SetWeightUnitsFetchState));
                self.link
                    .send_message(Msg::SetWeightUnitsFetchState(FetchAction::Fetching));
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
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link.send_message(Msg::ToggleAddFormDisplay(false));
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
                // Clear any fields which are not applicable to the currently selected publication type.
                // (Do not clear them before the save point as the user may change the type again.)
                if self.new_publication.publication_type != PublicationType::Paperback
                    && self.new_publication.publication_type != PublicationType::Hardback
                {
                    self.new_publication.weight = None;
                }
                let body = CreatePublicationRequestBody {
                    variables: Variables {
                        work_id: self.props.work_id,
                        publication_type: self.new_publication.publication_type.clone(),
                        isbn: self.new_publication.isbn.clone(),
                        weight: self.new_publication.weight,
                        units: self.props.weight_units_selection.clone(),
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
            Msg::ChangeWeight(value) => {
                self.new_publication.weight.neq_assign(value.to_opt_float())
            }
            Msg::ChangeWeightUnit(weight_unit) => {
                self.props.update_weight_units_selection.emit(weight_unit);
                // Callback will prompt parent to update this component's props.
                // This will trigger a re-render in change(), so not necessary
                // to also re-render here.
                false
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
            Msg::ToggleAddFormDisplay(true)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        // ISBNs cannot be added for publications whose work type is Book Chapter.
        let isbn_deactivated = self.props.work_type == WorkType::BookChapter;
        // Weight can only be added for physical (Paperback/Hardback) publications.
        let weight_deactivated = self.new_publication.publication_type
            != PublicationType::Paperback
            && self.new_publication.publication_type != PublicationType::Hardback;
        // Restrict the number of decimal places the user can enter for weight values
        // based on currently selected units.
        let step = match self.props.weight_units_selection {
            WeightUnit::G => "1".to_string(),
            WeightUnit::Oz => "0.0001".to_string(),
        };
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
                            <p class="modal-card-title">{ "New Publication" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="publications-form" onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreatePublication
                            })
                            >
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
                                    deactivated=isbn_deactivated
                                />
                                <FormFloatInput
                                    label = "Weight"
                                    value=self.new_publication.weight
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeWeight(e.value))
                                    step=step.clone()
                                    deactivated=weight_deactivated
                                />
                                <FormWeightUnitSelect
                                    label = "Units"
                                    value=self.props.weight_units_selection.clone()
                                    data=self.data.weight_units.clone()
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeWeightUnit(WeightUnit::from_str(&value).unwrap())
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="publications-form"
                            >
                                { "Add Publication" }
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

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Weight" }</label>
                        <div class="control is-expanded">
                            {&p.weight.as_ref().map(|w| w.to_string()).unwrap_or_else(|| "".to_string())}
                        </div>
                    </div>

                    <FormWeightUnitSelect
                        label = "Units"
                        value=self.props.weight_units_selection.clone()
                        data=self.data.weight_units.clone()
                        onchange=self.link.callback(|event| match event {
                            ChangeData::Select(elem) => {
                                let value = elem.value();
                                Msg::ChangeWeightUnit(WeightUnit::from_str(&value).unwrap())
                            }
                            _ => unreachable!(),
                        })
                        required = true
                    />

                    <div class="field is-grouped is-grouped-right">
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
