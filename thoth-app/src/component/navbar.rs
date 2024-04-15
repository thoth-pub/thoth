use thoth_api::account::model::AccountDetails;
use yew::html;
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew_router::prelude::*;

use crate::route::AppRoute;

pub struct NavbarComponent {}

pub enum Msg {
    Logout,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub current_user: Option<AccountDetails>,
    pub callback: Callback<()>,
}

impl Component for NavbarComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        NavbarComponent {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Logout => {
                ctx.props().callback.emit(());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> VNode {
        let logout = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Logout
        });
        html! {
            <nav class="navbar is-warning" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <a class="navbar-item" href="/">
                        <img src="https://cdn.thoth.pub/thoth_logo.png" width="50" height="58" style="max-height: none" />
                        <img src="https://cdn.thoth.pub/thoth_name.png" style="margin-left: 0.5em; margin-top: 0.5em" />
                    </a>

                    <a role="button" class="navbar-burger burger" aria-label="menu" aria-expanded="false" data-target="thothNavbar">
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    </a>
                </div>

                <div class="navbar-end">
                    <div class="navbar-item">
                        <div class="buttons">
                            <a class="version" href="https://github.com/thoth-pub/thoth/blob/master/CHANGELOG.md">
                                {"v"}{ env!("CARGO_PKG_VERSION") }
                            </a>
                            {
                                if ctx.props().current_user.is_some() {
                                    html! {
                                        <button class="button primary" onclick={ logout }>
                                            { "Logout" }
                                        </button>
                                    }
                                } else {
                                    html! {
                                        <Link<AppRoute> classes="button primary" to={ AppRoute::Login }>
                                            {"Login"}
                                        </Link<AppRoute>>
                                    }
                                }
                            }
                        </div>
                    </div>
                </div>
            </nav>
        }
    }
}
