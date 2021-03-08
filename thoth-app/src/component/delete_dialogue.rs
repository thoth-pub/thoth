use crate::string::CANCEL_BUTTON;
use crate::string::DELETE_BUTTON;
use yew::html;
use yew::prelude::*;

pub struct ConfirmDeleteComponent {
    props: Props,
    link: ComponentLink<Self>,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub onclick: Callback<MouseEvent>,
    pub object_name: String,
    #[prop_or(false)]
    pub show: bool,
}

pub enum Msg {
    ToggleConfirmDeleteDisplay(bool),
}

impl Component for ConfirmDeleteComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        ConfirmDeleteComponent { props, link }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleConfirmDeleteDisplay(value) => {
                self.props.show = value;
                true
            }
        }
    }

    fn view(&self) -> Html {
        let open_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleConfirmDeleteDisplay(true)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleConfirmDeleteDisplay(false)
        });
        html! {
            <>
                <button class="button is-danger" onclick=open_modal>
                    { DELETE_BUTTON }
                </button>
                <div class=self.confirm_delete_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "Confirm deletion" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <p>
                                { "Are you sure you want to delete " }
                                <i>{ &self.props.object_name }</i>
                                { "?" }
                            </p>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=&self.props.onclick
                            >
                                { DELETE_BUTTON }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
            </>
        }
    }
}

impl ConfirmDeleteComponent {
    fn confirm_delete_status(&self) -> String {
        match self.props.show {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }
}
