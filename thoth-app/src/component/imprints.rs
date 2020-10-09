use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::route::Route;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::models::imprint::imprints_query::FetchActionImprints;
use crate::models::imprint::imprints_query::FetchImprints;
use crate::models::imprint::Imprint;
use crate::component::utils::Loader;
use crate::component::utils::Reloader;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct ImprintsComponent {
    markdown: FetchImprints,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionImprints),
    GetMarkdown,
    ChangeRoute(AppRoute),
}

impl Component for ImprintsComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();

        ImprintsComponent {
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
            FetchState::NotFetching(_) => html! {<Reloader onclick=self.link.callback(|_| Msg::GetMarkdown)/>},
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => html! {
                <>
                    <nav class="level">
                        <div class="level-left">
                            <div class="level-item">
                                <p class="subtitle is-5">
                                    <strong>{ body.data.imprints.iter().count() }</strong> { " imprints" }
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
                                <th>{ "Imprint" }</th>
                                <th>{ "Publisher" }</th>
                                <th>{ "Imprint URL" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for body.data.imprints.iter().map(|p| self.render_imprint(p)) }
                        </tbody>
                    </table>
                </>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}

impl ImprintsComponent {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }

    fn render_imprint(&self, p: &Imprint) -> Html {
        html! {
            <tr
                class="row"
                onclick=&self.change_route(AppRoute::Admin(AdminRoute::Imprint(p.imprint_id.clone())))
            >
                <td>{&p.imprint_id}</td>
                <td>{&p.imprint_name}</td>
                <td>{&p.publisher.publisher_name}</td>
                <td>{&p.imprint_url.clone().unwrap_or("".to_string())}</td>
            </tr>
        }
    }
}
