#![allow(clippy::unnecessary_operation)]

use crate::string::CANCEL_BUTTON;
use crate::string::SAVE_BUTTON;
use yew::html;
use yew::prelude::*;

pub struct ConfirmWorkStatusComponent {
    show: bool,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onsubmit: Callback<()>,
    pub oncancel: Callback<()>,
    pub object_name: String,
    pub object_work_status: String,
    pub object_work_status_in_db: String,
}

pub enum Msg {
    CloseModal,
}

impl Component for ConfirmWorkStatusComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        ConfirmWorkStatusComponent { show: true }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CloseModal => {
                self.show = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::CloseModal
        });

        html! {
            <>
                <div class={ self.show_modal() }>
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
                                { "Are you sure you want to change the work status to " } { &ctx.props().object_work_status  } { " for " }
                                <i>{ &ctx.props().object_name }</i>
                                { "? Once a Work has been set to " } { &ctx.props().object_work_status }  { ", it is published and cannot be returned to the unpublished state of " } { &ctx.props().object_work_status_in_db }  { "." }
                            </p>
                        </section>

                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick={ ctx.link().callback({
                                    let onsubmit = ctx.props().onsubmit.clone();
                                    move |_| {
                                        onsubmit.emit(());
                                        Msg::CloseModal
                                    }
                                }) }
                            >
                                { SAVE_BUTTON }
                            </button>
                            <button
                                class="button"
                                onclick={ ctx.link().callback({
                                    let oncancel = ctx.props().oncancel.clone();
                                    move |_| {
                                        oncancel.emit(());
                                        Msg::CloseModal
                                    }
                                }) }
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
    fn show_modal(&self) -> String {
        match self.show {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }
}
