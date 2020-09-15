use yew::virtual_dom::VNode;
use yew::html;
use yew::Properties;
use yewtil::Pure;
use yewtil::PureComponent;

pub type Loader = Pure<PureLoader>;

#[derive(Clone, PartialEq, Properties)]
pub struct PureLoader {}

impl PureComponent for PureLoader {
    fn render(&self) -> VNode {
        html! {
            <div class="hero is-medium">
                <div class="hero-body">
                    <div class="container has-text-centered">
                        <progress class="progress is-warning" max="100"></progress>
                    </div>
                </div>
            </div>
        }
    }
}
