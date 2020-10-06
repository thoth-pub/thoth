use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::api::publications_query::FetchActionPublications;
use crate::api::publications_query::FetchPublications;
use crate::api::models::Publication;
use crate::component::utils::Loader;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct PublicationsComponent {
    markdown: FetchPublications,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionPublications),
    GetMarkdown,
    ChangeRoute(AppRoute),
}

impl Component for PublicationsComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();

        PublicationsComponent {
            markdown: Default::default(),
            link,
            router,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.markdown.fetch(Msg::SetMarkdownFetchState));
            self.link
                .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.markdown.apply(fetch_state);
                true
            }
            Msg::GetMarkdown => {
                self.link
                    .send_future(self.markdown.fetch(Msg::SetMarkdownFetchState));
                self.link
                    .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
                false
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
        match self.markdown.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {
                    <div class="buttons has-addons is-centered">
                        <button
                            class="button is-success is-large"
                            onclick=self.link.callback(|_| Msg::GetMarkdown)
                        >
                            <span class="icon">
                            <i class="fas fa-sync"></i>
                            </span>
                            <span>{"Reload"}</span>
                        </button>
                    </div>
                }
            }
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => html! {
                <>
                    <nav class="level">
                        <div class="level-left">
                            <div class="level-item">
                                <p class="subtitle is-5">
                                    <strong>{ body.data.publications.iter().count() }</strong> { " publications" }
                                </p>
                            </div>
                        </div>
                        <div class="level-right">
                            <p class="level-item">
                                <a class="button is-success">{ "New" }</a>
                            </p>
                        </div>
                    </nav>
                    <table class="table is-fullwidth is-hoverable">
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "Work Title" }</th>
                                <th>{ "Work DOI" }</th>
                                <th>{ "Publisher" }</th>
                                <th>{ "Type" }</th>
                                <th>{ "ISBN" }</th>
                                <th>{ "URL" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for body.data.publications.iter().map(|p| self.render_publication(p)) }
                        </tbody>
                    </table>
                </>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}

impl PublicationsComponent {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }

    fn render_publication(&self, p: &Publication) -> Html {
        html! {
            <tr
                class="row"
                onclick=&self.change_route(AppRoute::Admin(AdminRoute::Publication(p.publication_id.clone())))
            >
                <td>{&p.publication_id}</td>
                <td>{&p.work.title}</td>
                <td>{&p.work.doi.clone().unwrap_or("".to_string())}</td>
                <td>{&p.work.publisher()}</td>
                <td>{&p.publication_type}</td>
                <td>{&p.isbn.clone().unwrap_or("".to_string())}</td>
                <td>{&p.publication_url.clone().unwrap_or("".to_string())}</td>
            </tr>
        }
    }
}
