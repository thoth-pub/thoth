use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::prelude::RouterAnchor;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::component::utils::Loader;
use crate::component::utils::Reloader;
use crate::models::stats::stats_query::FetchActionStats;
use crate::models::stats::stats_query::FetchStats;
use crate::route::AdminRoute;
use crate::route::AppRoute;

pub struct DashboardComponent {
    get_stats: FetchStats,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetStatsFetchState(FetchActionStats),
    GetStats,
}

impl Component for DashboardComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DashboardComponent {
            get_stats: Default::default(),
            link,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.get_stats.fetch(Msg::SetStatsFetchState));
            self.link
                .send_message(Msg::SetStatsFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetStatsFetchState(fetch_state) => {
                self.get_stats.apply(fetch_state);
                true
            }
            Msg::GetStats => {
                self.link
                    .send_future(self.get_stats.fetch(Msg::SetStatsFetchState));
                self.link
                    .send_message(Msg::SetStatsFetchState(FetchAction::Fetching));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match self.get_stats.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {<Reloader onclick=self.link.callback(|_| Msg::GetStats)/>}
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
                                            {format!("{} Works", body.data.work_count)}
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
                                            {format!("{} Publications", body.data.publication_count)}
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
                                            {format!("{} Contributors", body.data.contributor_count)}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Contributors)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-info">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Publishers", body.data.publisher_count)}
                                        </p>
                                        <RouterAnchor<AppRoute>
                                            route=AppRoute::Admin(AdminRoute::Publications)
                                        >
                                            {"See all"}
                                        </  RouterAnchor<AppRoute>>
                                    </div>
                                </article>
                            </div>
                        </div>
                        <div class="tile">
                            <div class="tile is-parent is-vertical">
                                <article class="tile is-child notification is-danger">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Series", body.data.series_count)}
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
                                            {format!("{} Imprints", body.data.imprint_count)}
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
