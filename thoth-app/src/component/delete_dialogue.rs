use crate::string::CANCEL_BUTTON;
use crate::string::DELETE_BUTTON;
use yew::html;
use yew::prelude::*;

pub struct ConfirmDeleteComponent {}

#[derive(PartialEq, Properties)]
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

    fn create(ctx: &Context<Self>) -> Self {
        ConfirmDeleteComponent {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleConfirmDeleteDisplay(value) => {
                ctx.props().show = value;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleConfirmDeleteDisplay(true)
        });
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleConfirmDeleteDisplay(false)
        });
        html! {
            <>
                <button class="button is-danger" onclick={ open_modal }>
                    { DELETE_BUTTON }
                </button>
                <div class={ self.confirm_delete_status(ctx) }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "Confirm deletion" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <p>
                                { "Are you sure you want to delete " }
                                <i>{ &ctx.props().object_name }</i>
                                { "?" }
                            </p>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick={ &ctx.props().onclick }
                            >
                                { DELETE_BUTTON }
                            </button>
                            <button
                                class="button"
                                onclick={ &close_modal }
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
    fn confirm_delete_status(&self, ctx: &Context<Self>) -> String {
        match ctx.props().show {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }
}
