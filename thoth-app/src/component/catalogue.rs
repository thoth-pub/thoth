use thoth_api::work::model::WorkWithRelations;
use yew::html;
use yew::prelude::Component;
use yew::prelude::FocusEvent;
use yew::prelude::Html;
use yew::prelude::InputData;
use yew::prelude::ShouldRender;
use yew::ComponentLink;
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
use crate::models::work::DisplayWork;

pub struct CatalogueComponent {
    limit: i32,
    offset: i32,
    page_size: i32,
    search_term: String,
    data: Vec<WorkWithRelations>,
    result_count: i32,
    fetch_data: FetchWorks,
    link: ComponentLink<Self>,
}

pagination_helpers! {CatalogueComponent, PAGINATION_COUNT_WORKS, SEARCH_WORKS}

pub enum Msg {
    SetFetchState(FetchActionWorks),
    GetData,
    PaginateData,
    #[allow(dead_code)]
    Search(String),
    ChangeSearchTerm(String),
    TriggerSearch,
    NextPage,
    PreviousPage,
}

impl Component for CatalogueComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let offset: i32 = Default::default();
        let page_size: i32 = 10;
        let limit: i32 = page_size;
        let search_term: String = Default::default();
        let result_count: i32 = Default::default();
        let data = Default::default();
        let fetch_data = Default::default();

        link.send_message(Msg::PaginateData);

        CatalogueComponent {
            limit,
            offset,
            page_size,
            search_term,
            data,
            result_count,
            fetch_data,
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetFetchState(fetch_state) => {
                self.fetch_data.apply(fetch_state);
                self.data = match self.fetch_data.as_ref().state() {
                    FetchState::Fetched(body) => body.data.works.clone(),
                    _ => Default::default(),
                };
                self.result_count = match self.fetch_data.as_ref().state() {
                    FetchState::Fetched(body) => body.data.work_count,
                    _ => Default::default(),
                };
                true
            }
            Msg::GetData => {
                self.link
                    .send_future(self.fetch_data.fetch(Msg::SetFetchState));
                self.link
                    .send_message(Msg::SetFetchState(FetchAction::Fetching));
                false
            }
            Msg::PaginateData => {
                let filter = self.search_term.clone();
                let body = WorksRequestBody {
                    variables: Variables {
                        limit: Some(self.limit),
                        offset: Some(self.offset),
                        filter: Some(filter),
                        // Sorting option is not required on Catalogue page
                        order: None,
                        // Catalogue is public so results should never be filtered by logged-in user
                        publishers: None,
                    },
                    ..Default::default()
                };
                let request = WorksRequest { body };
                self.fetch_data = Fetch::new(request);
                self.link.send_message(Msg::GetData);
                false
            }
            Msg::Search(_) => {
                // needed because of macro, but unused here
                false
            }
            Msg::ChangeSearchTerm(term) => {
                self.search_term = term;
                false
            }
            Msg::TriggerSearch => {
                self.limit = self.page_size;
                self.offset = 0;
                self.link.send_message(Msg::PaginateData);
                false
            }
            Msg::NextPage => {
                if self.limit < self.result_count && !self.is_next_disabled() {
                    self.offset += self.page_size;
                    self.link.send_message(Msg::PaginateData);
                }
                false
            }
            Msg::PreviousPage => {
                if self.offset > 0 && !self.is_previous_disabled() {
                    self.offset -= self.page_size;
                    self.link.send_message(Msg::PaginateData);
                }
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="container">
                <h1 class="title">{ "Catalogue" }</h1>
                <nav class="level">
                    <div class="level-left">
                        <p class="level-item">
                            <span>
                            { self.display_count() }
                            </span>
                        </p>
                    </div>
                    <div class="level-right" />
                </nav>
                <nav class="pagination is-centered" role="navigation" aria-label="pagination">
                    <a class="pagination-previous"
                        onclick=self.link.callback(|_| Msg::PreviousPage)
                        disabled=self.is_previous_disabled()
                    >{ crate::string::PREVIOUS_PAGE_BUTTON }</a>
                    <a class="pagination-next"
                        onclick=self.link.callback(|_| Msg::NextPage)
                        disabled=self.is_next_disabled()
                    >{ crate::string::NEXT_PAGE_BUTTON }</a>
                    <div class="pagination-list">
                        <form
                            style="width: 80%"
                            onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::TriggerSearch
                            })
                        >
                            <div class="field has-addons">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        value=self.search_term.clone()
                                        placeholder=self.search_text()
                                        oninput=self.link.callback(|e: InputData| Msg::ChangeSearchTerm(e.value))
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                                <div class="control">
                                    <button class="button is-info" type="submit">
                                        { "Search" }
                                    </button>
                                </div>
                            </div>
                        </form>
                    </div>
                </nav>
                {
                    match self.fetch_data.as_ref().state() {
                        FetchState::NotFetching(_) => {
                            html! {<Reloader onclick=self.link.callback(|_| Msg::GetData)/>}
                        }
                        FetchState::Fetching(_) => html! {<Loader/>},
                        FetchState::Fetched(_body) => html! {
                            { for self.data.iter().map(|w| w.as_catalogue_box()) }
                        },
                        FetchState::Failed(_, err) => html! {&err},
                    }
                }
            </div>
        }
    }
}
