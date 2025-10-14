#![allow(clippy::unnecessary_operation)]

use gloo_timers::callback::Timeout;
use thoth_api::model::contributor::Contributor;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;

use crate::models::contributor::contributors_query::ContributorsRequest;
use crate::models::contributor::contributors_query::ContributorsRequestBody;
use crate::models::contributor::contributors_query::FetchActionContributors;
use crate::models::contributor::contributors_query::FetchContributors;
use crate::models::contributor::contributors_query::Variables;
use crate::models::Dropdown;
use crate::DEFAULT_DEBOUNCING_TIMEOUT;

use super::ToElementValue;

pub struct ContributorSelectComponent {
    contributors: Vec<Contributor>,
    fetch_contributors: FetchContributors,
    search_callback: Callback<()>,
    search_query: String,
    debounce_timeout: Option<Timeout>,
    show_results: bool,
}

pub enum Msg {
    SetContributorsFetchState(FetchActionContributors),
    GetContributors,
    SearchQueryChanged(String),
    SearchContributor,
    ToggleSearchResultDisplay(bool),
    SelectContributor(Contributor),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub callback: Callback<Contributor>,
}

impl Component for ContributorSelectComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let contributors: Vec<Contributor> = Default::default();
        let body = ContributorsRequestBody {
            variables: Variables {
                limit: Some(100),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = ContributorsRequest { body };
        let fetch_contributors = Fetch::new(request);
        let search_callback = ctx.link().callback(|_| Msg::SearchContributor);
        let search_query: String = Default::default();
        let debounce_timeout: Option<Timeout> = None;
        let show_results = false;

        ctx.link().send_message(Msg::GetContributors);

        ContributorSelectComponent {
            contributors,
            fetch_contributors,
            search_callback,
            search_query,
            debounce_timeout,
            show_results,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetContributorsFetchState(fetch_state) => {
                self.fetch_contributors.apply(fetch_state);
                self.contributors = match self.fetch_contributors.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.contributors.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetContributors => {
                ctx.link().send_future(
                    self.fetch_contributors
                        .fetch(Msg::SetContributorsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetContributorsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SearchQueryChanged(value) => {
                self.search_query = value;
                // cancel previous timeout
                self.debounce_timeout = self.debounce_timeout.take().and_then(|timeout| {
                    timeout.cancel();
                    None
                });

                if !self.search_query.is_empty() {
                    // start new timeout
                    let search_callback = self.search_callback.clone();
                    let timeout = Timeout::new(DEFAULT_DEBOUNCING_TIMEOUT, move || {
                        search_callback.emit(());
                    });
                    self.debounce_timeout = Some(timeout);
                } else {
                    self.contributors = Default::default();
                }
                false
            }
            Msg::SearchContributor => {
                let body = ContributorsRequestBody {
                    variables: Variables {
                        filter: Some(self.search_query.clone()),
                        limit: Some(25),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = ContributorsRequest { body };
                self.fetch_contributors = Fetch::new(request);
                ctx.link().send_message(Msg::GetContributors);
                false
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SelectContributor(contributor) => {
                ctx.props().callback.emit(contributor);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let show_results = self.show_results && !self.contributors.is_empty();
        let dropdown_status = match show_results {
            true => "dropdown is-active".to_string(),
            false => "dropdown".to_string(),
        };

        html! {
            <div class={ dropdown_status } style="width: 100%">
                <div class="dropdown-trigger" style="width: 100%">
                    <div class="field">
                        <p class="control is-expanded has-icons-left">
                            <input
                                class="input"
                                type="search"
                                placeholder="Search Contributor"
                                aria-haspopup="true"
                                aria-controls="contributors-menu"
                                oninput={ ctx.link().callback(|e: InputEvent| Msg::SearchQueryChanged(e.to_value())) }
                                onfocus={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(true)) }
                                onblur={ ctx.link().callback(|_| Msg::ToggleSearchResultDisplay(false)) }
                            />
                            <span class="icon is-left">
                                <i class="fas fa-search" aria-hidden="true"></i>
                            </span>
                        </p>
                    </div>
                </div>
            {
                if show_results {
                    html! {
                        <div class="dropdown-menu" id="contributors-menu" role="menu">
                            <div class="dropdown-content">
                                {
                                    for self.contributors.iter().map(|i| {
                                        let contributor = i.clone();
                                        i.as_dropdown_item(
                                            ctx.link().callback(move |_| {
                                                Msg::SelectContributor(contributor.clone())
                                            })
                                        )
                                    })
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            </div>
        }
    }
}
