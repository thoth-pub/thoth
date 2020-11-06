use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;

use crate::component::utils::Reloader;
use crate::models::work::works_query::FetchActionWorks;
use crate::models::work::works_query::FetchWorks;

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
                    { for body.data.works.iter().map(|w| w.as_catalogue_box()) }
                </div>
            },
            FetchState::Failed(_, err) => html! {&err},
        }
    }
}
