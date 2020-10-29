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
use crate::models::work::works_query::FetchActionWorks;
use crate::models::work::works_query::FetchWorks;
use crate::models::work::works_query::Variables;
use crate::models::work::works_query::WorksRequest;
use crate::models::work::works_query::WorksRequestBody;
use crate::models::work::Work;
use crate::route::AdminRoute;
use crate::route::AppRoute;
use crate::string::NEXT_PAGE_BUTTON;
use crate::string::PAGINATION_COUNT_WORKS;
use crate::string::PREVIOUS_PAGE_BUTTON;

pub struct WorksComponent {
    limit: i32,
    offset: i32,
    page_size: i32,
    search_term: String,
    works: Vec<Work>,
    result_count: i32,
    fetch_works: FetchWorks,
    link: ComponentLink<Self>,
    router: RouteAgentDispatcher<()>,
}

pub enum Msg {
    SetWorksFetchState(FetchActionWorks),
    GetWorks,
    PaginateWorks,
    SearchWorks(String),
    NextPage,
    PreviousPage,
    ChangeRoute(AppRoute),
}

impl Component for WorksComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let router = RouteAgentDispatcher::new();
        let offset = 0;
        let page_size = 20;
        let limit = page_size;
        let search_term = "".into();
        let result_count = 0;
        let works = vec![];

        link.send_message(Msg::PaginateWorks);

        WorksComponent {
            limit,
            offset,
            page_size,
            search_term,
            works,
            result_count,
            fetch_works: Default::default(),
            link,
            router,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetWorksFetchState(fetch_state) => {
                self.fetch_works.apply(fetch_state);
                self.works = match self.fetch_works.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.works.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                self.result_count = match self.fetch_works.as_ref().state() {
                    FetchState::NotFetching(_) => 0,
                    FetchState::Fetching(_) => 0,
                    FetchState::Fetched(body) => body.data.work_count,
                    FetchState::Failed(_, _err) => 0,
                };
                true
            }
            Msg::GetWorks => {
                self.link
                    .send_future(self.fetch_works.fetch(Msg::SetWorksFetchState));
                self.link
                    .send_message(Msg::SetWorksFetchState(FetchAction::Fetching));
                false
            }
            Msg::PaginateWorks => {
                let filter = self.search_term.clone();
                let body = WorksRequestBody {
                    variables: Variables {
                        limit: Some(self.limit),
                        offset: Some(self.offset),
                        filter: Some(filter),
                    },
                    ..Default::default()
                };
                let request = WorksRequest { body };
                self.fetch_works = Fetch::new(request);
                self.link.send_message(Msg::GetWorks);
                false
            }
            Msg::SearchWorks(term) => {
                self.limit = self.page_size;
                self.offset = 0;
                self.search_term = term;
                self.link.send_message(Msg::PaginateWorks);
                false
            }
            Msg::NextPage => {
                if self.limit < self.result_count && !self.is_next_disabled() {
                    self.limit += self.page_size;
                    self.offset += self.page_size;
                    self.link.send_message(Msg::PaginateWorks);
                }
                false
            }
            Msg::PreviousPage => {
                if self.offset > 0 && !self.is_previous_disabled() {
                    self.limit -= self.page_size;
                    self.offset -= self.page_size;
                    self.link.send_message(Msg::PaginateWorks);
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
                        <p class="level-item">
                            <span>
                            { self.display_count() }
                            </span>
                        </p>
                    </div>
                    <div class="level-right">
                        <p class="level-item">
                                <RouterAnchor<AppRoute>
                                    classes="button is-success"
                                    route=AppRoute::Admin(AdminRoute::NewWork)
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
                                    placeholder="Search by title, DOI, internal reference, abstract or landing page"
                                    oninput=self.link.callback(|e: InputData| Msg::SearchWorks(e.value))
                                />
                                <span class="icon is-left">
                                    <i class="fas fa-search" aria-hidden="true"></i>
                                </span>
                            </p>
                        </div>
                    </div>
                </nav>
                {
                    match self.fetch_works.as_ref().state() {
                        FetchState::NotFetching(_) => {
                            html! {<Reloader onclick=self.link.callback(|_| Msg::GetWorks)/>}
                        },
                        FetchState::Fetching(_) => html! {<Loader/>},
                        FetchState::Fetched(_body) => html! {
                            <table class="table is-fullwidth is-hoverable">
                                <thead>
                                    <tr>
                                        <th>{ "ID" }</th>
                                        <th>{ "Title" }</th>
                                        <th>{ "Type" }</th>
                                        <th>{ "Contributors" }</th>
                                        <th>{ "DOI" }</th>
                                        <th>{ "Publisher" }</th>
                                    </tr>
                                </thead>

                                <tbody>
                                    {
                                        for self.works.iter().map(|w| {
                                            let route = w.edit_route().clone();
                                            w.as_table_row(
                                                self.link.callback(move |_| {
                                                    Msg::ChangeRoute(route.clone())
                                                })
                                            )
                                        })
                                    }
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

impl WorksComponent {
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
            PAGINATION_COUNT_WORKS, offset_display, limit_display, self.result_count
        )
    }

    fn is_previous_disabled(&self) -> bool {
        self.offset < self.page_size
    }

    fn is_next_disabled(&self) -> bool {
        self.limit >= self.result_count
    }
}
