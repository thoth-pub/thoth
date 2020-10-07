use std::str::FromStr;
use thoth_api::models::publication::PublicationType;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::api::publication_types_query::FetchActionPublicationTypes;
use crate::api::publication_types_query::FetchPublicationTypes;
use crate::api::models::Publication;
use crate::api::models::PublicationTypeValues;
use crate::component::utils::FormTextInput;
use crate::component::utils::FormUrlInput;
use crate::component::utils::FormPublicationTypeSelect;
use crate::string::EMPTY_PUBLICATIONS;

pub struct PublicationsFormComponent {
    props: Props,
    data: PublicationsFormData,
    new_publication: Publication,
    show_add_form: bool,
    fetch_publication_types: FetchPublicationTypes,
    link: ComponentLink<Self>,
}

struct PublicationsFormData {
    publication_types: Vec<PublicationTypeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetPublicationTypesFetchState(FetchActionPublicationTypes),
    GetPublicationTypes,
    ChangePublicationType(PublicationType),
    ChangeIsbn(String),
    ChangeUrl(String),
    AddPublication,
    RemovePublication(String),
    DoNothing,
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
            publication_types: vec![],
        };
        let show_add_form = false;
        let new_publication: Publication = Default::default();

        link.send_message(Msg::GetPublicationTypes);

        PublicationsFormComponent {
            props,
            data,
            new_publication,
            show_add_form,
            fetch_publication_types: Default::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetPublicationTypesFetchState(fetch_state) => {
                self.fetch_publication_types.apply(fetch_state);
                self.data.publication_types = match self.fetch_publication_types.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.publication_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetPublicationTypes => {
                self.link.send_future(
                    self.fetch_publication_types
                        .fetch(Msg::SetPublicationTypesFetchState),
                );
                self.link
                    .send_message(Msg::SetPublicationTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::ChangePublicationType(val) => self.new_publication.publication_type.neq_assign(val),
            Msg::ChangeIsbn(isbn) => self.new_publication.isbn.neq_assign(Some(isbn)),
            Msg::ChangeUrl(url) => self.new_publication.publication_url.neq_assign(Some(url)),
            Msg::AddPublication => {
                let publication = self.new_publication.clone();
                let mut publications: Vec<Publication> = self.props.publications.clone().unwrap_or_default();
                let publication = Publication {
                    publication_id: publication.publication_id,
                    work_id: self.props.work_id.clone(),
                    publication_type: publication.publication_type,
                    isbn: publication.isbn,
                    publication_url: publication.publication_url,
                };
                publications.push(publication);
                self.new_publication = Default::default();
                self.props.update_publications.emit(Some(publications));
                self.link.send_message(Msg::ToggleAddFormDisplay(false));
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
            Msg::DoNothing => false,  // callbacks need to return a message
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let publications = self.props.publications.clone().unwrap_or_else(|| vec![]);
        let open_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(true)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Publications" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick=open_modal
                    >
                        { "Add Publication" }
                    </button>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Publication" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::DoNothing
                            })
                            >
                                <FormPublicationTypeSelect
                                    label = "Publication Type"
                                    value=&self.new_publication.publication_type
                                    data=&self.data.publication_types
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangePublicationType(
                                                PublicationType::from_str(&value).unwrap()
                                            )
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormTextInput
                                    label = "ISBN"
                                    value=&self.new_publication.isbn.clone().unwrap_or("".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeIsbn(e.value))
                                />
                                <FormUrlInput
                                    label = "URL"
                                    value=&self.new_publication.publication_url.clone().unwrap_or("".to_string())
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeUrl(e.value))
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::AddPublication
                                })
                            >
                                { "Add Publication" }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { "Cancel" }
                            </button>
                        </footer>
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
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
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
