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
use crate::component::prices_form::PricesFormComponent;
use crate::component::utils::Loader;
use crate::models::price::Price;
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
use crate::models::publication::Publication;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::DELETE_BUTTON;

pub struct PublicationComponent {
    publication: Publication,
    fetch_publication: FetchPublication,
    delete_publication: PushDeletePublication,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
    notification_bus: NotificationDispatcher,
}

pub enum Msg {
    SetPublicationFetchState(FetchActionPublication),
    GetPublication,
    SetPublicationDeleteState(PushActionDeletePublication),
    DeletePublication,
    UpdatePrices(Option<Vec<Price>>),
    ChangeRoute(AppRoute),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub publication_id: String,
}

impl Component for PublicationComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let body = PublicationRequestBody {
            variables: Variables {
                publication_id: Some(props.publication_id),
            },
            ..Default::default()
        };
        let request = PublicationRequest { body };
        let fetch_publication = Fetch::new(request);
        let delete_publication = Default::default();
        let notification_bus = NotificationBus::dispatcher();
        let publication: Publication = Default::default();
        let router = RouteAgentDispatcher::new();

        link.send_message(Msg::GetPublication);

        PublicationComponent {
            publication,
            fetch_publication,
            delete_publication,
            link,
            router,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
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
                        true
                    }
                    FetchState::Failed(_, _err) => false,
                }
            }
            Msg::GetPublication => {
                self.link
                    .send_future(self.fetch_publication.fetch(Msg::SetPublicationFetchState));
                self.link
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
                                format!("Deleted {}", &p.isbn.clone().unwrap_or(p.publication_id.clone())),
                                NotificationStatus::Success,
                            )));
                            self.link.send_message(Msg::ChangeRoute(AppRoute::Admin(
                                AdminRoute::Publications,
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
            Msg::DeletePublication => {
                let body = DeletePublicationRequestBody {
                    variables: DeleteVariables {
                        publication_id: self.publication.publication_id.clone(),
                    },
                    ..Default::default()
                };
                let request = DeletePublicationRequest { body };
                self.delete_publication = Fetch::new(request);
                self.link
                    .send_future(self.delete_publication.fetch(Msg::SetPublicationDeleteState));
                self.link
                    .send_message(Msg::SetPublicationDeleteState(FetchAction::Fetching));
                false
            }
            Msg::UpdatePrices(prices) => {
                self.publication.prices.neq_assign(prices)
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
                                    <button class="button is-danger" onclick=self.link.callback(|_| Msg::DeletePublication)>
                                        { DELETE_BUTTON }
                                    </button>
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
                                    {&self.publication.isbn.clone().unwrap_or_else(|| "".to_string())}
                                </div>
                            </div>

                            <div class="field">
                                <label class="label">{ "Publication URL" }</label>
                                <div class="control is-expanded">
                                    {&self.publication.publication_url.clone().unwrap_or_else(|| "".to_string())}
                                </div>
                            </div>
                            <hr/>
                            <article class="message is-info">
                                <div class="message-body">
                                    { "Prices below are saved automatically upon change." }
                                </div>
                            </article>

                            <PricesFormComponent
                                prices=&self.publication.prices
                                publication_id=&self.publication.publication_id
                                update_prices=self.link.callback(|p: Option<Vec<Price>>| Msg::UpdatePrices(p))
                            />
                        </form>
                    </>
                }
            }
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
