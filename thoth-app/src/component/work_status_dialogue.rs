use crate::string::CANCEL_BUTTON;
use crate::string::SAVE_BUTTON;
use thoth_api::account::model::AccountDetails;
use yew::html;
use yew::prelude::*;

pub struct ConfirmWorkStatusComponent {
    show: bool,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onclick: Option<Callback<MouseEvent>>,
    pub object_name: String,
    pub current_user: AccountDetails,
    pub current_state_unpublished: bool,
    pub is_published: bool,
    // pub form_callback: Callback<()>,
    #[prop_or_default]
    pub deactivated: bool,
}

pub enum Msg {
    ToggleConfirmWorkStatusDisplay(bool),
    ExecuteCallback
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
            Msg::ExecuteCallback => {
                self.show = false;
                // trigger the callback 
                // _ctx.props().form_callback(|_| form_callback).emit(());
                // form_callback.emit(());
                // when set as true, the modal closes and it saves correctly
                // true
                // when set as false, the modal also closes and saves correctly. 
                
                false
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
        let modal_behavior = if !ctx.props().current_user.resource_access.is_superuser
            && ctx.props().current_state_unpublished
            && ctx.props().is_published {
                &open_modal
            } else {
                &close_modal
            };
        
        html! {
            <>
                <button
                    class="button is-success"
                    // onclick={ open_modal }
                    onclick={ modal_behavior }
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
                                { "? Once a Work has been set to Active, it cannot be set back to Forthcoming." }
                            </p>
                        </section>
                        
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick={ ctx.props().onclick.clone() }
                                // onclick={ ctx.link().callback(|_| Msg::ExecuteCallback) }
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
