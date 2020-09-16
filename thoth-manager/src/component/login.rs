use yew::html;
use yew::prelude::*;

use crate::string::TEXT_LOGIN;
use crate::string::INPUT_EMAIL;
use crate::string::INPUT_PASSWORD;

pub struct LoginComponent {}

impl Component for LoginComponent {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        LoginComponent {}
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="columns is-mobile is-centered">
                <div class="column is-3">
                    <div class="box">
                        <div class="field">
                            <p class="control has-icons-left has-icons-right">
                                <input class="input" type="email" placeholder=INPUT_EMAIL />
                                <span class="icon is-small is-left">
                                    <i class="fas fa-envelope"></i>
                                </span>
                                <span class="icon is-small is-right">
                                    <i class="fas fa-check"></i>
                                </span>
                            </p>
                        </div>
                        <div class="field">
                            <p class="control has-icons-left">
                                <input class="input" type="password" placeholder=INPUT_PASSWORD />
                                <span class="icon is-small is-left">
                                    <i class="fas fa-lock"></i>
                                </span>
                            </p>
                        </div>
                        <div class="field">
                            <p class="control">
                                <button class="button is-success">
                                    { TEXT_LOGIN }
                                </button>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
