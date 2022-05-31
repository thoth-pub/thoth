use thoth_api::model::publication::Publication;
use thoth_api::model::publication::PublicationProperties;
use thoth_api::model::work::WorkType;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::publication_modal::PublicationModalComponent;
use crate::models::publication::delete_publication_mutation::DeletePublicationRequest;
use crate::models::publication::delete_publication_mutation::DeletePublicationRequestBody;
use crate::models::publication::delete_publication_mutation::PushActionDeletePublication;
use crate::models::publication::delete_publication_mutation::PushDeletePublication;
use crate::models::publication::delete_publication_mutation::Variables as DeleteVariables;
use crate::models::EditRoute;
use crate::route::AppRoute;
use crate::string::EDIT_BUTTON;
use crate::string::EMPTY_PUBLICATIONS;
use crate::string::REMOVE_BUTTON;
use crate::string::VIEW_BUTTON;

pub struct PublicationsFormComponent {
    show_modal_form: bool,
    publication_under_edit: Option<Publication>,
    delete_publication: PushDeletePublication,
    notification_bus: NotificationDispatcher,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    ToggleModalFormDisplay(bool, Option<Publication>),
    AddPublication(Publication),
    UpdatePublication(Publication),
    SetPublicationDeleteState(PushActionDeletePublication),
    DeletePublication(Uuid),
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

    fn create(ctx: &Context<Self>) -> Self {
        let show_modal_form = false;
        let publication_under_edit = Default::default();
        let delete_publication = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let router = RouteAgentDispatcher::new();

        PublicationsFormComponent {
            show_modal_form,
            publication_under_edit,
            delete_publication,
            notification_bus,
            router,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleModalFormDisplay(show_form, p) => {
                self.show_modal_form = show_form;
                self.publication_under_edit = p;
                true
            }
            Msg::AddPublication(p) => {
                // Child form has created a new publication - add it to list
                let mut publications: Vec<Publication> =
                    ctx.props().publications.clone().unwrap_or_default();
                publications.push(p);
                ctx.props().update_publications.emit(Some(publications));
                // Close child form
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(false, None));
                true
            }
            Msg::UpdatePublication(p) => {
                // Child form has updated an existing publication - replace it in list
                let mut publications: Vec<Publication> =
                    ctx.props().publications.clone().unwrap_or_default();
                if let Some(publication) = publications
                    .iter_mut()
                    .find(|pb| pb.publication_id == p.publication_id)
                {
                    *publication = p.clone();
                    ctx.props().update_publications.emit(Some(publications));
                } else {
                    // This should not be possible: the updated publication returned from the
                    // database does not match any of the locally-stored publication data.
                    // Refreshing the page will reload the local data from the database.
                    self.notification_bus.send(Request::NotificationBusMsg((
                        "Changes were saved but display failed to update. Refresh your browser to view current data.".to_string(),
                        NotificationStatus::Warning,
                    )));
                }
                // Close child form
                ctx.link()
                    .send_message(Msg::ToggleModalFormDisplay(false, None));
                true
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
                            ctx.props().update_publications.emit(Some(to_keep));
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
                ctx.link().send_future(
                    self.delete_publication
                        .fetch(Msg::SetPublicationDeleteState),
                );
                ctx.link()
                    .send_message(Msg::SetPublicationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeRoute(r) => {
                let route = Route::from(r);
                self.router.send(RouteRequest::ChangeRoute(route));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let publications = ctx.props().publications.clone().unwrap_or_default();
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleModalFormDisplay(true, None)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Publications" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick={ open_modal }
                    >
                        { "Add Publication" }
                    </button>
                </div>
                {
                    if !publications.is_empty() {
                        html!{{for publications.iter().map(|p| self.render_publication(ctx, p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_PUBLICATIONS }
                            </div>
                        }
                    }
                }
                <PublicationModalComponent
                    publication_under_edit={ self.publication_under_edit.clone() }
                    work_id={ ctx.props().work_id }
                    work_type={ ctx.props().work_type.clone() }
                    show_modal_form={ self.show_modal_form }
                    add_publication={ ctx.link().callback(Msg::AddPublication) }
                    update_publication={ ctx.link().callback(Msg::UpdatePublication) }
                    close_modal_form={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(false, None)) }
                />
            </nav>
        }
    }
}

impl PublicationsFormComponent {
    fn render_publication(&self, ctx: &Context<Self>, p: &Publication) -> Html {
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
                        if p.is_physical() && ctx.props().work_type != WorkType::BookChapter {
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
                                onclick={ ctx.link().callback(move |_| Msg::ToggleModalFormDisplay(true, Some(publication.clone()))) }
                            >
                                { EDIT_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-info"
                                onclick={ ctx.link().callback(move |_| Msg::ChangeRoute(route.clone())) }
                            >
                                { VIEW_BUTTON }
                            </a>
                        </div>
                        <div class="control">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeletePublication(publication_id)) }
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
