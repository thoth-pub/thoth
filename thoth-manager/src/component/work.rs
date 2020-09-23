use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::api::work_query::WORK_QUERY;
use crate::api::work_query::Variables;
use crate::api::work_query::FetchWork;
use crate::api::work_query::FetchActionWork;
use crate::api::work_query::WorkRequest;
use crate::api::work_query::WorkRequestBody;
use crate::component::work_form::WorkFormComponent;
use crate::component::utils::Loader;

pub struct WorkComponent {
    fetch_work: FetchWork,
    link: ComponentLink<Self>,
}

pub enum Msg {
    SetMarkdownFetchState(FetchActionWork),
    GetWork,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub work_id: String,
}

impl Component for WorkComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let body = WorkRequestBody {
            query: WORK_QUERY.to_string(),
            variables: Variables { work_id: Some(props.work_id) },
        };
        let request = WorkRequest { body };
        let fetch_work = Fetch::new(request);

        WorkComponent {
            fetch_work,
            link,
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.link
                .send_future(self.fetch_work.fetch(Msg::SetMarkdownFetchState));
            self.link
                .send_message(Msg::SetMarkdownFetchState(FetchAction::Fetching));
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetMarkdownFetchState(fetch_state) => {
                self.fetch_work.apply(fetch_state);
                true
            }
            Msg::GetWork => {
                self.link
                    .send_future(self.fetch_work.fetch(Msg::SetMarkdownFetchState));
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
        match self.fetch_work.as_ref().state() {
            FetchState::NotFetching(_) => {
                html! {
                    <div class="buttons has-addons is-centered">
                        <button
                            class="button is-success is-large"
                            onclick=self.link.callback(|_| Msg::GetWork)
                        >
                            <span class="icon">
                            <i class="fas fa-sync"></i>
                            </span>
                            <span>{"Reload"}</span>
                        </button>
                    </div>
                }
            }
            FetchState::Fetching(_) => html! {<Loader/>},
            FetchState::Fetched(body) => {
                if let Some(w) = &body.data.work {
                    html! {
                        <WorkFormComponent
                            work = w
                            imprints = &body.data.imprints
                            work_types = &body.data.work_types.enum_values
                            work_statuses = &body.data.work_statuses.enum_values
                        />
                    }
                } else {
                    html!{{ "Work could not be found" }}
                }
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
