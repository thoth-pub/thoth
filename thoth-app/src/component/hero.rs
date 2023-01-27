use yew::function_component;
use yew::html;

use crate::{THOTH_EXPORT_API, THOTH_GRAPHQL_API};

#[function_component(HeroComponent)]
pub fn hero_component() -> VNode {
    let graphiql = format!("{THOTH_GRAPHQL_API}/graphiql");
    html! {
        <section class="hero is-warning">
            <div class="hero-body">
                <div class="container has-text-centered">
                    <img class="home-banner" src="/img/thoth-banner.png" height="200" />
                </div>

                <nav class="columns home-icons">
                    <a class="home-icons-item column has-text-centered" href={graphiql}>
                        <p class="title is-4">
                            <strong>{ "Open API" }</strong>
                        </p>
                        <p class="subtitle is-6">
                            { "Try it with " }<strong>{ "GraphiQL" }</strong>
                        </p>
                        <figure>
                            <span class="icon is-large">
                                <i class="fas fa-code fa-3x" aria-hidden="true"></i>
                            </span>
                        </figure>
                    </a>

                    <a class="home-icons-item column has-text-centered" href={THOTH_EXPORT_API}>
                        <p class="title is-4">
                            <strong>{ "Open Standards" }</strong>
                        </p>
                        <p class="subtitle is-6">
                            { "Read the " }<strong>{ "Export API" }</strong>{ " docs" }
                        </p>
                        <figure>
                            <span class="icon is-large">
                                <i class="fas fa-file-export fa-3x" aria-hidden="true"></i>
                            </span>
                        </figure>
                    </a>

                    <a class="home-icons-item column has-text-centered" href="https://doi.org/10.21428/785a6451.eb0d86e8">
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
