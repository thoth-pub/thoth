use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::api::detailed_publications_query::DetailedPublicationsRequest;
use crate::api::detailed_publications_query::DetailedPublicationsRequestBody;
use crate::api::detailed_publications_query::FetchActionDetailedPublications;
use crate::api::detailed_publications_query::FetchDetailedPublications;
use crate::api::detailed_publications_query::Variables;
use crate::api::detailed_publications_query::DETAILED_PUBLICATIONS_QUERY;
use crate::api::models::DetailedPublication;
use crate::api::models::Publication;
use crate::string::EMPTY_PUBLICATIONS;

pub struct PublicationsFormComponent {
    props: Props,
    data: PublicationsFormData,
    show_results: bool,
    fetch_publications: FetchDetailedPublications,
    link: ComponentLink<Self>,
}

struct PublicationsFormData {
    publications: Vec<DetailedPublication>,
}

pub enum Msg {
    SetPublicationsFetchState(FetchActionDetailedPublications),
    GetPublications,
    ToggleSearchResultDisplay(bool),
    SearchPublication(String),
    AddPublication(DetailedPublication),
    RemovePublication(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub publications: Option<Vec<Publication>>,
    pub work_id: String,
    pub update_publications: Callback<Option<Vec<Publication>>>,
}

impl Component for PublicationsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data = PublicationsFormData {
            publications: vec![],
        };
        let show_results = false;

        link.send_message(Msg::GetPublications);

        PublicationsFormComponent {
            props,
            data,
            show_results,
            fetch_publications: Default::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetPublicationsFetchState(fetch_state) => {
                self.fetch_publications.apply(fetch_state);
                self.data.publications = match self.fetch_publications.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.publications.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetPublications => {
                self.link.send_future(
                    self.fetch_publications
                        .fetch(Msg::SetPublicationsFetchState),
                );
                self.link
                    .send_message(Msg::SetPublicationsFetchState(FetchAction::Fetching));
                false
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SearchPublication(value) => {
                let body = DetailedPublicationsRequestBody {
                    query: DETAILED_PUBLICATIONS_QUERY.to_string(),
                    variables: Variables {
                        work_id: None,
                        contributor_id: None,
                        filter: Some(value),
                    },
                };
                let request = DetailedPublicationsRequest { body };
                self.fetch_publications = Fetch::new(request);
                self.link.send_message(Msg::GetPublications);
                false
            }
            Msg::AddPublication(publication) => {
                let mut publications: Vec<Publication> = self.props.publications.clone().unwrap_or_default();
                let publication = Publication {
                    publication_id: publication.publication_id,
                    work_id: self.props.work_id.clone(),
                    publication_type: publication.publication_type,
                    isbn: publication.isbn,
                    publication_url: publication.publication_url,
                };
                publications.push(publication);
                self.props.update_publications.emit(Some(publications));
                true
            }
            Msg::RemovePublication(publication_id) => {
                let to_keep: Vec<Publication> = self
                    .props
                    .publications
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|i| i.publication_id != publication_id)
                    .collect();
                self.props.update_publications.emit(Some(to_keep));
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let publications = self.props.publications.clone().unwrap_or_else(|| vec![]);
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Publications" }
                </p>
                <div class="panel-block">
                    <div class=self.search_dropdown_status() style="width: 100%">
                        <div class="dropdown-trigger" style="width: 100%">
                            <div class="field">
                                <p class="control is-expanded has-icons-left">
                                    <input
                                        class="input"
                                        type="search"
                                        placeholder="Search Publication"
                                        aria-haspopup="true"
                                        aria-controls="publications-menu"
                                        oninput=self.link.callback(|e: InputData| Msg::SearchPublication(e.value))
                                        onfocus=self.link.callback(|_| Msg::ToggleSearchResultDisplay(true))
                                        onblur=self.link.callback(|_| Msg::ToggleSearchResultDisplay(false))
                                    />
                                    <span class="icon is-left">
                                        <i class="fas fa-search" aria-hidden="true"></i>
                                    </span>
                                </p>
                            </div>
                        </div>
                        <div class="dropdown-menu" id="publications-menu" role="menu">
                            <div class="dropdown-content">
                                { for self.data.publications.iter().map(|c| self.render_publications(c)) }
                            </div>
                        </div>
                    </div>
                </div>
                {
                    if publications.len() > 0 {
                        html!{{for publications.iter().map(|p| self.render_publication(p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_PUBLICATIONS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl PublicationsFormComponent {
    fn search_dropdown_status(&self) -> String {
        match self.show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        }
    }

    fn render_publications(&self, p: &DetailedPublication) -> Html {
        let publication = p.clone();
        // avoid listing publications already present in contributions list
        if let Some(_index) = self.props.publications
            .as_ref()
            .unwrap()
            .iter()
            .position(|pu| pu.publication_id == p.publication_id)
        {
            html! {}
        } else {
            // since publications dropdown has an onblur event, we need to use onmousedown instead of
            // onclick. This is not ideal, but it seems to be the only event that'd do the calback
            // without disabling onblur so that onclick can take effect
            html! {
                <div
                    onmousedown=self.link.callback(move |_| Msg::AddPublication(publication.clone()))
                    class="dropdown-item"
                >
                { format!("{} ({}, {})", &p.work.doi.clone().unwrap_or(p.work.title.clone()), &p.publication_type, p.isbn.clone().unwrap_or("".to_string())) }
                </div>
            }
        }
    }

    fn render_publication(&self, p: &Publication) -> Html {
        // there's probably a better way to do this. We basically need to copy 3 instances
        // of contributor_id and take ownership of them so they can be passed on to
        // the callback functions
        let publication_id = p.publication_id.clone();
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
                            {&p.isbn.clone().unwrap_or("".to_string())}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "URL" }</label>
                        <div class="control is-expanded">
                            {&p.publication_url.clone().unwrap_or("".to_string())}
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::RemovePublication(publication_id.clone()))
                            >
                                { "Remove" }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
