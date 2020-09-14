use serde::Deserialize;
use serde::Serialize;
use serde::de::{self, Deserializer};
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::{Fetch, FetchAction, FetchRequest, FetchState, Json, MethodBody};
use yewtil::future::LinkFuture;

pub struct CatalogueComponent {
    markdown: Fetch<Request, ResponseBody>,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchAction<ResponseBody>),
    GetMarkdown,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum LicenseType {
    By,
    BySa,
    ByNd,
    ByNc,
    ByNcSa,
    ByNcNd,
    Zero,
}

impl<'de> Deserialize<'de> for LicenseType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let l = String::deserialize(deserializer)?.to_lowercase();
        let license = match l.as_str() {
            "http://creativecommons.org/licenses/by/1.0/"
                | "http://creativecommons.org/licenses/by/2.0/"
                | "http://creativecommons.org/licenses/by/2.5/"
                | "http://creativecommons.org/licenses/by/3.0/"
                | "http://creativecommons.org/licenses/by/4.0/" => LicenseType::By,
            "http://creativecommons.org/licenses/by-sa/1.0/"
                  | "http://creativecommons.org/licenses/by-sa/2.0/"
                  | "http://creativecommons.org/licenses/by-sa/2.5/"
                  | "http://creativecommons.org/licenses/by-sa/3.0/"
                  | "http://creativecommons.org/licenses/by-sa/4.0/" => LicenseType::BySa,
            "http://creativecommons.org/licenses/by-nd/1.0/"
                  | "http://creativecommons.org/licenses/by-nd/2.0/"
                  | "http://creativecommons.org/licenses/by-nd/2.5/"
                  | "http://creativecommons.org/licenses/by-nd/3.0/"
                  | "http://creativecommons.org/licenses/by-nd/4.0/" => LicenseType::ByNd,
            "http://creativecommons.org/licenses/by-nc/1.0/"
                  | "http://creativecommons.org/licenses/by-nc/2.0/"
                  | "http://creativecommons.org/licenses/by-nc/2.5/"
                  | "http://creativecommons.org/licenses/by-nc/3.0/"
                  | "http://creativecommons.org/licenses/by-nc/4.0/" => LicenseType::ByNc,
            "http://creativecommons.org/licenses/by-nc-sa/1.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/2.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/2.5/"
                  | "http://creativecommons.org/licenses/by-nc-sa/3.0/"
                  | "http://creativecommons.org/licenses/by-nc-sa/4.0/" => LicenseType::ByNcSa,
            "http://creativecommons.org/licenses/by-nc-nd/1.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/2.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/2.5/"
                  | "http://creativecommons.org/licenses/by-nc-nd/3.0/"
                  | "http://creativecommons.org/licenses/by-nc-nd/4.0/" => LicenseType::ByNcNd,
            "https://creativecommons.org/publicdomain/zero/1.0/" => LicenseType::Zero,
            other => { return Err(de::Error::custom(format!("Invalid license '{}'", other))); },
        };
        Ok(license)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Request {
    body: RequestBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    work_id: String,
    full_title: String,
    cover_url: String,
    license: LicenseType,
    doi: String,
    publication_date: Option<String>,
    place: String,
    contributions: Option<Vec<Contribution>>,
    imprint: Imprint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Imprint {
    publisher: Publisher,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Publisher {
    publisher_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contribution {
    main_contribution: bool,
    contributor: Contributor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contributor {
    full_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseBody {
    data: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseData {
    works: Vec<Work>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestBody {
    query: String,
    variables: String,
}

impl Default for ResponseBody {
    fn default() -> ResponseBody {
        ResponseBody {
            data: ResponseData { works: vec![] },
        }
    }
}

impl Default for ResponseData {
    fn default() -> ResponseData {
        ResponseData {
            works: vec![],
        }
    }
}

impl Default for RequestBody {
    fn default() -> RequestBody {
        RequestBody {
            query: "
                {
                works(limit: 9999) {
                    fullTitle
                    workId
                    coverUrl
                    license
                    doi
                    publicationDate
                    place
                    contributions {
                        mainContribution
                        contributor {
                            fullName
                        }
                    }
                    imprint {
                        publisher {
                            publisherName
                        }
                    }
                }
            }
            ".to_string(),
            variables: "null".to_string()
        }
    }
}

impl FetchRequest for Request {
    type RequestBody = RequestBody;
    type ResponseBody = ResponseBody;
    type Format = Json;

    fn url(&self) -> String {
        "http://localhost:8000/graphql".to_string()
    }

    fn method(&self) -> MethodBody<Self::RequestBody> {
        MethodBody::Post(&self.body)
    }

    fn headers(&self) -> Vec<(String, String)> {
        vec![("Content-Type".to_string(), "application/json".to_string())]
    }

    fn use_cors(&self) -> bool {
        true
    }
}

impl Component for CatalogueComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        CatalogueComponent {
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
            FetchState::Fetching(_) => html! {
                <div class="pageloader is-active is-warning">
                    <span class="title">{ "Loading" }</span>
                 </div>
            },
            FetchState::Fetched(body) => html! {
                <div class="container">
                    { for body.data.works.iter().map(render_work) }
                </div>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}

fn render_contribution(c: &Contribution) -> Html {
    if c.main_contribution {
        html! {
            <small class="contributor">
                {&c.contributor.full_name}
                <span>{ " • " }</span>
            </small>
        }
    } else {
        html! {}
    }
}

fn render_license(license: &LicenseType) -> Html {
    html! {
        <span class="icon is-small license">
            <i class="fab fa-creative-commons" aria-hidden="true"></i>
            {
                match license {
                    LicenseType::By =>html!{
                        <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                    },
                    LicenseType::BySa => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-sa" aria-hidden="true"></i>
                        </>
                    },
                    LicenseType::ByNd => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nd" aria-hidden="true"></i>
                        </>
                    },
                    LicenseType::ByNc => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nc" aria-hidden="true"></i>
                        </>
                    },
                    LicenseType::ByNcSa => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nc" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-sa" aria-hidden="true"></i>
                        </>
                    },
                    LicenseType::ByNcNd => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nc" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nd" aria-hidden="true"></i>
                        </>
                    },
                    LicenseType::Zero => html!{
                        <i class="fab fa-creative-commons-zero" aria-hidden="true"></i>
                    }
                }
            }
        </span>
    }
}

fn render_work(w: &Work) -> Html {
    html! {
        <div class="box">
            <article class="media">
                <div class="media-left">
                <figure class="image is-96x96">
                    <img src={&w.cover_url} alt="Placeholder image" />
                    { render_license(&w.license) }
                </figure>
                </div>
                <div class="media-content">
                    <div class="content">
                        <p>
                            <strong>{&w.full_title}</strong>
                            <br/>
                            <div>
                            {
                                if let Some(contributions) = &w.contributions {
                                    contributions.iter().map(render_contribution).collect::<Html>()
                                } else {
                                    html! {}
                                }
                            }
                            </div>
                            <br/>
                            {
                                if let Some(date) = &w.publication_date {
                                    let mut c1 = date.chars();
                                    c1.next();
                                    c1.next();
                                    c1.next();
                                    c1.next();
                                    let year: &str = &date[..date.len() - c1.as_str().len()];
                                    html! {<small>{&w.place}{": "}{&w.imprint.publisher.publisher_name}{", "}{year}</small>}
                                } else {
                                    html! {<small>{&w.imprint.publisher.publisher_name}</small>}
                                }
                            }
                            <br/>
                            <small>{&w.doi}</small>
                        </p>
                    </div>
                    <nav class="level is-mobile">
                        <div class="level-left">
                            <a
                                class="level-item button is-small"
                                aria-label="read"
                                href={format!("{}", &w.doi)}
                            >
                                <span class="icon is-small">
                                <i class="fas fa-book" aria-hidden="true"></i>
                                </span>
                                <span>{"Read"}</span>
                            </a>

                            <div class="level-item dropdown is-hoverable">
                                <div class="dropdown-trigger">
                                    <button
                                        class="button is-small"
                                        aria-haspopup="true"
                                        aria-controls="dropdown-menu"
                                    >
                                        <span class="icon is-small">
                                            <i class="fas fa-file" aria-hidden="true"></i>
                                        </span>
                                        <span>{"Metadata"} </span>
                                        <span class="icon is-small">
                                            <i class="fas fa-angle-down" aria-hidden="true"></i>
                                        </span>
                                    </button>
                                </div>
                                <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                    <div class="dropdown-content">
                                        <a
                                            href={format!("http://localhost:8000/onix/{}", &w.work_id)}
                                            class="dropdown-item"
                                        >
                                        {"ONIX"}
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </nav>
                </div>
            </article>
        </div>
    }
}
