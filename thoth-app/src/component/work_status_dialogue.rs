use crate::string::CANCEL_BUTTON;
use crate::string::SAVE_BUTTON;
use yew::html;
use yew::prelude::*;

pub struct ConfirmWorkStatusComponent {
    show: bool,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onclick: Option<Callback<MouseEvent>>,
    pub object_name: String,
    #[prop_or_default]
    pub deactivated: bool,
}

pub enum Msg {
    ToggleConfirmWorkStatusDisplay(bool),
}

impl Component for ConfirmWorkStatusComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        ConfirmWorkStatusComponent { show: false }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleConfirmWorkStatusDisplay(value) => {
                self.show = value;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleConfirmWorkStatusDisplay(true)
        });
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleConfirmWorkStatusDisplay(false)
        });
        html! {
            <>
                <button
                    class="button is-success"
                    onclick={ open_modal }
                    disabled={ ctx.props().deactivated }
                >
                    { SAVE_BUTTON }
                </button>
                <div class={ self.confirm_work_status_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "Confirm changing work status" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <p>
                                { "Are you sure you want to change the work status to Active for " }
                                <i>{ &ctx.props().object_name }</i>
                                { "?" }
                            </p>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick={ ctx.props().onclick.clone() }
                            >
                                { SAVE_BUTTON }
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

impl ConfirmWorkStatusComponent {
    fn confirm_work_status_status(&self) -> String {
        match self.show {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }
}