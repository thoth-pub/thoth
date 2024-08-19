use gloo_timers::callback::Timeout;
use thoth_api::model::institution::Institution;
use yew::html;
use yew::prelude::*;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;

use crate::models::institution::institutions_query::FetchActionInstitutions;
use crate::models::institution::institutions_query::FetchInstitutions;
use crate::models::institution::institutions_query::InstitutionsRequest;
use crate::models::institution::institutions_query::InstitutionsRequestBody;
use crate::models::institution::institutions_query::Variables;
use crate::models::Dropdown;
use crate::DEFAULT_DEBOUNCING_TIMEOUT;

use super::ToElementValue;

pub struct InstitutionSelectComponent {
    institutions: Vec<Institution>,
    fetch_institutions: FetchInstitutions,
    search_callback: Callback<()>,
    search_query: String,
    debounce_timeout: Option<Timeout>,
    show_results: bool,
}

pub enum Msg {
    SetInstitutionsFetchState(FetchActionInstitutions),
    GetInstitutions,
    SearchQueryChanged(String),
    SearchInstitution,
    ToggleSearchResultDisplay(bool),
    SelectInstitution(Institution),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub callback: Callback<Institution>,
}

impl Component for InstitutionSelectComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let institutions: Vec<Institution> = Default::default();
        let body = InstitutionsRequestBody {
            variables: Variables {
                limit: Some(100),
                ..Default::default()
            },
            ..Default::default()
        };
        let request = InstitutionsRequest { body };
        let fetch_institutions = Fetch::new(request);
        let search_callback = ctx.link().callback(|_| Msg::SearchInstitution);
        let search_query: String = Default::default();
        let debounce_timeout: Option<Timeout> = None;
        let show_results = false;

        ctx.link().send_message(Msg::GetInstitutions);

        InstitutionSelectComponent {
            institutions,
            fetch_institutions,
            search_callback,
            search_query,
            debounce_timeout,
            show_results,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetInstitutionsFetchState(fetch_state) => {
                self.fetch_institutions.apply(fetch_state);
                self.institutions = match self.fetch_institutions.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.institutions.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetInstitutions => {
                ctx.link().send_future(
                    self.fetch_institutions
                        .fetch(Msg::SetInstitutionsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetInstitutionsFetchState(FetchAction::Fetching));
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
                    self.institutions = Default::default();
                }
                false
            }
            Msg::SearchInstitution => {
                let body = InstitutionsRequestBody {
                    variables: Variables {
                        filter: Some(self.search_query.clone()),
                        limit: Some(25),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = InstitutionsRequest { body };
                self.fetch_institutions = Fetch::new(request);
                ctx.link().send_message(Msg::GetInstitutions);
                false
            }
            Msg::ToggleSearchResultDisplay(value) => {
                self.show_results = value;
                true
            }
            Msg::SelectInstitution(institution) => {
                ctx.props().callback.emit(institution);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let show_results = self.show_results && !self.institutions.is_empty();
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
                                placeholder="Search Institution"
                                aria-haspopup="true"
                                aria-controls="institutions-menu"
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
                        <div class="dropdown-menu" id="institutions-menu" role="menu">
                            <div class="dropdown-content">
                                {
                                    for self.institutions.iter().map(|i| {
                                        let institution = i.clone();
                                        i.as_dropdown_item(
                                            ctx.link().callback(move |_| {
                                                Msg::SelectInstitution(institution.clone())
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
