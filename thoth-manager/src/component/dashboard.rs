use yew::ComponentLink;
use yew::html;
use yew::prelude::*;

pub struct DashboardComponent {}

impl Component for DashboardComponent {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DashboardComponent{}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
        <>
            <section class="header"> {"Admin Dashboard"} </section>
        </>
        }
    }
}
