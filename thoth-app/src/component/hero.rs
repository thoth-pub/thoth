use yew::html;
use yew::virtual_dom::VNode;
use yew::Properties;
use yewtil::Pure;
use yewtil::PureComponent;

use crate::THOTH_API;

pub type HeroComponent = Pure<PureHero>;

#[derive(Clone, PartialEq, Properties)]
pub struct PureHero {}

impl PureComponent for PureHero {
    fn render(&self) -> VNode {
        html! {
            <section class="hero is-warning">
                <div class="hero-body">
                    <div class="container has-text-centered">
                        <img class="home-banner" src="/img/thoth-banner.png" height="200" />
                    </div>

                    <nav class="columns home-icons">
                        <a class="home-icons-item column has-text-centered" href={format!("{}/graphiql", THOTH_API)}>
                            <p class="title is-4">
                                <strong>{ "Open API" }</strong>
                            </p>
                            <p class="subtitle is-6">
                                { "Try the " }<strong>{ "GraphiQL" }</strong>{ " playground" }
                            </p>
                            <figure>
                                <span class="icon is-large">
                                    <i class="fas fa-code fa-3x" aria-hidden="true"></i>
                                </span>
                            </figure>
                        </a>

                        <a class="home-icons-item column has-text-centered" href="https://creativecommons.org/publicdomain/zero/1.0/">
                            <p class="title is-4">
                                <strong>{ "Open Metadata" }</strong>
                            </p>
                            <p class="subtitle is-6">
                                { "Truly "}<strong>{ "open data" }</strong>
                            </p>
                            <figure>
                                <span class="icon is-large">
                                    <i class="fab fa-creative-commons fa-3x" aria-hidden="true"></i>
                                </span>
                                <span class="icon is-large">
                                    <i class="fab fa-creative-commons-zero fa-3x" aria-hidden="true"></i>
                                </span>
                            </figure>
                        </a>

                        <a class="home-icons-item column has-text-centered" href="https://www.copim.ac.uk/">
                            <p class="title is-4">
                                <strong>{ "Open Access" }</strong>
                            </p>
                            <p class="subtitle is-6">
                                { "Built for " }<strong>{ "OA" }</strong>{ " books" }
                            </p>
                            <figure>
                                <span class="icon is-large">
                                    <i class="fas fa-lock-open fa-3x" aria-hidden="true"></i>
                                </span>
                            </figure>
                        </a>

                        <a class="home-icons-item column has-text-centered" href="https://github.com/thoth-pub/thoth/" target="_blank">
                            <p class="title is-4">
                                <strong>{ "Open Source" }</strong>
                            </p>
                            <p class="subtitle is-6">
                                { "Code available on "}<strong>{ "GitHub" }</strong>
                            </p>
                            <figure>
                                <span class="icon is-large">
                                    <i class="fab fa-github fa-3x" aria-hidden="true"></i>
                                </span>
                            </figure>
                        </a>
                    </nav>

                </div>
            </section>
        }
    }
}
