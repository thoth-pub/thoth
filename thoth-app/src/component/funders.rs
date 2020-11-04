use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yew_router::agent::RouteAgentDispatcher;
use yew_router::agent::RouteRequest;
use yew_router::prelude::RouterAnchor;
use yew_router::route::Route;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::component::utils::Loader;
use crate::component::utils::Reloader;
use crate::models::funder::funders_query::FetchActionFunders;
use crate::models::funder::funders_query::FetchFunders;
use crate::models::funder::funders_query::Variables;
use crate::models::funder::funders_query::FundersRequest;
use crate::models::funder::funders_query::FundersRequestBody;
use crate::models::funder::Funder;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::NEXT_PAGE_BUTTON;
use crate::string::PAGINATION_COUNT_FUNDERS;
use crate::string::PREVIOUS_PAGE_BUTTON;
use crate::string::SEARCH_FUNDERS;

pub struct FundersComponent {
    limit: i32,
    offset: i32,
    page_size: i32,
    search_term: String,
    funders: Vec<Funder>,
    result_count: i32,
    fetch_funders: FetchFunders,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetFundersFetchState(FetchActionFunders),
    GetFunders,
    PaginateFunders,
    SearchFunders(String),
    NextPage,
    PreviousPage,
    ChangeRoute(AppRoute),
}

impl Component for FundersComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router: RouteAgentDispatcher<()> = RouteAgentDispatcher::new();
        let offset: i32 = Default::default();
        let page_size: i32 = 20;
        let limit: i32 = Default::default();
        let search_term: String = Default::default();
        let result_count: i32 = Default::default();
        let funders: Vec<Funder> = Default::default();
        let fetch_funders: FetchFunders = Default::default();

        FundersComponent {
            limit,
            offset,
            page_size,
            search_term,
            funders,
            result_count,
            fetch_funders,
            link,
            router,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.fetch_funders.fetch(Msg::SetFundersFetchState));
            self.link
                .send_message(Msg::SetFundersFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFundersFetchState(fetch_state) => {
                self.fetch_funders.apply(fetch_state);
                self.funders = match self.fetch_funders.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.funders.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                self.result_count = match self.fetch_funders.as_ref().state() {
                    FetchState::NotFetching(_) => 0,
                    FetchState::Fetching(_) => 0,
                    FetchState::Fetched(body) => body.data.funder_count,
                    FetchState::Failed(_, _err) => 0,
                };
                true
            }
            Msg::GetFunders => {
                self.link
                    .send_future(self.fetch_funders.fetch(Msg::SetFundersFetchState));
                self.link
                    .send_message(Msg::SetFundersFetchState(FetchAction::Fetching));
                false
            }
            Msg::PaginateFunders => {
                let filter = self.search_term.clone();
                let body = FundersRequestBody {
                    variables: Variables {
                        limit: Some(self.limit),
                        offset: Some(self.offset),
                        filter: Some(filter),
                    },
                    ..Default::default()
                };
                let request = FundersRequest { body };
                self.fetch_funders = Fetch::new(request);
                self.link.send_message(Msg::GetFunders);
                false
            }
            Msg::SearchFunders(term) => {
                self.limit = self.page_size;
                self.offset = 0;
                self.search_term = term;
                self.link.send_message(Msg::PaginateFunders);
                false
            }
            Msg::NextPage => {
                if self.limit < self.result_count && !self.is_next_disabled() {
                    self.limit += self.page_size;
                    self.offset += self.page_size;
                    self.link.send_message(Msg::PaginateFunders);
                }
                false
            }
            Msg::PreviousPage => {
                if self.offset > 0 && !self.is_previous_disabled() {
                    self.limit -= self.page_size;
                    self.offset -= self.page_size;
                    self.link.send_message(Msg::PaginateFunders);
                }
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
        html! {
            <>
                <nav class="level">
                    <div class="level-left">
                        <div class="level-item">
                            <p class="level-item">
                                <span>
                                { self.display_count() }
                                </span>
                            </p>
                        </div>
                    </div>
                    <div class="level-right">
                        <p class="level-item">
                            <RouterAnchor<AppRoute>
                                classes="button is-success"
                                route=AppRoute::Admin(AdminRoute::NewFunder)
                            >
                                {"New"}
                            </  RouterAnchor<AppRoute>>
                        </p>
                    </div>
                </nav>
                <nav class="pagination is-centered" role="navigation" aria-label="pagination">
                    <a class="pagination-previous"
                        onclick=self.link.callback(|_| Msg::PreviousPage)
                        disabled=self.is_previous_disabled()
                    >{ PREVIOUS_PAGE_BUTTON }</a>
                    <a class="pagination-next"
                        onclick=self.link.callback(|_| Msg::NextPage)
                        disabled=self.is_next_disabled()
                    >{ NEXT_PAGE_BUTTON }</a>
                    <div class="pagination-list">
                        <div class="field" style="width: 80%">
                            <p class="control is-expanded has-icons-left">
                                <input
                                    class="input"
                                    type="search"
                                    value=self.search_term
                                    placeholder=SEARCH_FUNDERS
                                    oninput=self.link.callback(|e: InputData| Msg::SearchFunders(e.value))
                                />
                                <span class="icon is-left">
                                    <i class="fas fa-search" aria-hidden="true"></i>
                                </span>
                            </p>
                        </div>
                    </div>
                </nav>
                {
                    match self.fetch_funders.as_ref().state() {
                        FetchState::NotFetching(_) => {
                            html! {<Reloader onclick=self.link.callback(|_| Msg::GetFunders)/>}
                        },
                        FetchState::Fetching(_) => html! {<Loader/>},
                        FetchState::Fetched(_body) => html! {
                            <table class="table is-fullwidth is-hoverable">
                                <thead>
                                    <tr>
                                        <th>{ "ID" }</th>
                                        <th>{ "Funder" }</th>
                                        <th>{ "DOI" }</th>
                                    </tr>
                                </thead>

                                <tbody>
                                    { for self.funders.iter().map(|p| self.render_funder(p)) }
                                </tbody>
                            </table>
                        },
                        FetchState::Failed(_, err) => html! {&err},
                    }
                }
            </>
        }
    }
}

impl FundersComponent {
    fn change_route(&self, app_route: AppRoute) -> Callback<MouseEvent> {
        self.link.callback(move |_| {
            let route = app_route.clone();
            Msg::ChangeRoute(route)
        })
    }

    fn display_count(&self) -> String {
        let offset_display = match self.offset == 0 && self.result_count > 0 {
            true => 1,
            false => self.offset,
        };
        let limit_display = match self.limit > self.result_count {
            true => self.result_count,
            false => self.limit,
        };
        format!(
            "{} {}-{} of {}",
            PAGINATION_COUNT_FUNDERS, offset_display, limit_display, self.result_count
        )
    }

    fn is_previous_disabled(&self) -> bool {
        self.offset < self.page_size
    }

    fn is_next_disabled(&self) -> bool {
        self.limit >= self.result_count
    }

    fn render_funder(&self, f: &Funder) -> Html {
        html! {
            <tr
                class="row"
                onclick=&self.change_route(AppRoute::Admin(AdminRoute::Funder(f.funder_id.clone())))
            >
                <td>{&f.funder_id}</td>
                <td>{&f.funder_name}</td>
                <td>{&f.funder_doi.clone().unwrap_or("".to_string())}</td>
            </tr>
        }
    }
}
