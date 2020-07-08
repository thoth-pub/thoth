use yew::prelude::*;

pub struct RootComponent {
    link: ComponentLink<Self>,
    value: i64,
}

pub enum Msg {
    AddOne,
}

impl Component for RootComponent {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => self.value += 1
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
        <>
            <section class="header">
                <div class="container">
                    <img class="logo" src="/thoth-logo.png" />
                    <a class="button" href="https://github.com/OpenBookPublishers/thoth" title="Project">{ "Project" }</a>
                    <a class="button" href="https://github.com/orgs/OpenBookPublishers/projects/1" title="Timeline">{ "Timeline" }</a>
                    <a class="button" href="https://github.com/OpenBookPublishers/thoth/blob/master/roadmap.md" title="Timeline">{ "Roadmap" }</a>
                    <a class="button" href="/graphiql" title="GraphiQL">{ "GraphiQL" }</a>
                </div>
            </section>
            <div class="container works">
                <div id="works" class="center list flex-column"></div>
            </div>
        </>
        }
    }
}
