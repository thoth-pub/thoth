use thoth_api::account::model::AccountAccess;
use thoth_api::account::model::AccountDetails;
use thoth_errors::ThothError;
use yew::html;
use yew::prelude::*;
use yew_router::prelude::Link;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::component::utils::Loader;
use crate::component::utils::Reloader;
use crate::models::stats::stats_query::FetchActionStats;
use crate::models::stats::stats_query::FetchStats;
use crate::models::stats::stats_query::StatsRequest;
use crate::models::stats::stats_query::StatsRequestBody;
use crate::models::stats::stats_query::Variables;
use crate::route::AdminRoute;

pub struct DashboardComponent {
    get_stats: FetchStats,
    // Store props value locally in order to test whether it has been updated on props change
    resource_access: AccountAccess,
}

pub enum Msg {
    SetStatsFetchState(FetchActionStats),
    GetStats,
}

#[derive(PartialEq, Eq, Properties)]
pub struct Props {
    pub current_user: AccountDetails,
}

impl Component for DashboardComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::GetStats);

        DashboardComponent {
            get_stats: Default::default(),
            resource_access: ctx.props().current_user.resource_access.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetStatsFetchState(fetch_state) => {
                self.get_stats.apply(fetch_state);
                true
            }
            Msg::GetStats => {
                let body = StatsRequestBody {
                    variables: Variables {
                        publishers: ctx.props().current_user.resource_access.restricted_to(),
                    },
                    ..Default::default()
                };
                let request = StatsRequest { body };
                self.get_stats = Fetch::new(request);

                ctx.link()
                    .send_future(self.get_stats.fetch(Msg::SetStatsFetchState));
                ctx.link()
                    .send_message(Msg::SetStatsFetchState(FetchAction::Fetching));
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let updated_permissions = self
            .resource_access
            .neq_assign(ctx.props().current_user.resource_access.clone());
        if updated_permissions {
            ctx.link().send_message(Msg::GetStats);
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.get_stats.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {<Reloader onclick={ ctx.link().callback(|_| Msg::GetStats) }/>}
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
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Works }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-info">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Books", body.data.book_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Books }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-danger">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Chapters", body.data.chapter_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Chapters }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                            </div>
                        </div>
                        <div class="tile">
                            <div class="tile is-parent is-vertical">
                                <article class="tile is-child notification is-link">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Publications", body.data.publication_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Publications }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-warning">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Contributors", body.data.contributor_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Contributors }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-info">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Publishers", body.data.publisher_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Publishers }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
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
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Serieses }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-success">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Imprints", body.data.imprint_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Imprints }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                                <article class="tile is-child notification is-warning">
                                    <div class="content">
                                        <p class="title">
                                            {format!("{} Institutions", body.data.institution_count)}
                                        </p>
                                        <Link<AdminRoute>
                                            to={ AdminRoute::Institutions }
                                        >
                                            {"See all"}
                                        </Link<AdminRoute>>
                                    </div>
                                </article>
                            </div>
                        </div>
                    </div>
                </div>
            },
            FetchState::Failed(_, err) => html! {
                { ThothError::from(err).to_string() }
            },
        }
    }
}
