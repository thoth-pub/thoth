use yew::ComponentLink;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yew_router::prelude::RouterAnchor;

use crate::api::stats_query::FetchStats;
use crate::api::stats_query::FetchActionStats;
use crate::component::utils::Loader;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct DashboardComponent {
    markdown: FetchStats,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionStats),
    GetMarkdown,
}

impl Component for DashboardComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DashboardComponent{
            markdown: Default::default(),
            link,
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
                <div class="tile is-ancestor">
                    <div class="tile">
                        <div class="tile">
                            <div class="tile is-parent is-vertical">
                                <article class="tile is-child notification is-primary">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Works", body.data.works.iter().count())}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Works)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-link">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Publishers", body.data.publishers.iter().count())}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Publishers)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                            </div>
                        </div>
                        <div class="tile">
                            <div class="tile is-parent is-vertical">
                                <article class="tile is-child notification is-warning">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Contributors", body.data.contributors.iter().count())}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Contributors)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-danger">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Series", body.data.serieses.iter().count())}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Serieses)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-success">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Imprints", body.data.imprints.iter().count())}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Imprints)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                            </div>
                        </div>
                    </div>
                </div>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
