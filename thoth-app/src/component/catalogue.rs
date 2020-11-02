use std::str::FromStr;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::component::utils::Reloader;
use crate::models::work::works_query::FetchActionWorks;
use crate::models::work::works_query::FetchWorks;
use crate::models::work::License;
use crate::models::work::Work;
use crate::THOTH_API;

pub struct CatalogueComponent {
    markdown: FetchWorks,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionWorks),
    GetMarkdown,
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
                html! {<Reloader onclick=self.link.callback(|_| Msg::GetMarkdown)/>}
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

fn render_license(license: &License) -> Html {
    html! {
        <span class="icon is-small license">
            <i class="fab fa-creative-commons" aria-hidden="true"></i>
            {
                match license {
                    License::By =>html!{
                        <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                    },
                    License::BySa => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-sa" aria-hidden="true"></i>
                        </>
                    },
                    License::ByNd => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nd" aria-hidden="true"></i>
                        </>
                    },
                    License::ByNc => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nc" aria-hidden="true"></i>
                        </>
                    },
                    License::ByNcSa => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nc" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-sa" aria-hidden="true"></i>
                        </>
                    },
                    License::ByNcNd => html!{
                        <>
                            <i class="fab fa-creative-commons-by" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nc" aria-hidden="true"></i>
                            <i class="fab fa-creative-commons-nd" aria-hidden="true"></i>
                        </>
                    },
                    License::Zero => html!{
                        <i class="fab fa-creative-commons-zero" aria-hidden="true"></i>
                    },
                    License::Undefined => html! {}
                }
            }
        </span>
    }
}

fn render_work(w: &Work) -> Html {
    let doi = &w.doi.clone().unwrap_or_else(|| "".to_string());
    let license = License::from_str(&w.license.clone().unwrap_or_else(|| "".to_string())).unwrap();
    let cover_url = &w.cover_url.clone().unwrap_or_else(|| "".to_string());
    let place = &w.place.clone().unwrap_or_else(|| "".to_string());
    html! {
        <div class="box">
            <article class="media">
                <div class="media-left">
                <figure class="image is-96x96">
                    <img src={cover_url} alt={format!("{} - Cover Image", &w.title)} loading="lazy" />
                    { render_license(&license) }
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
                                    contributions.iter().map(|c| c.main_contribution_item_bullet_small()).collect::<Html>()
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
                                    html! {<small>{place}{": "}{&w.imprint.publisher.publisher_name}{", "}{year}</small>}
                                } else {
                                    html! {<small>{&w.imprint.publisher.publisher_name}</small>}
                                }
                            }
                            <br/>
                            <small>{&doi}</small>
                        </p>
                    </div>
                    <nav class="level is-mobile">
                        <div class="level-left">
                            <a
                                class="level-item button is-small"
                                aria-label="read"
                                href={format!("{}", doi)}
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
                                            href={format!("{}/onix/{}", THOTH_API, &w.work_id)}
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
