use yew::html;
use yew::prelude::*;

pub struct AboutComponent {}

impl Component for AboutComponent {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        AboutComponent {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="container">
                <h1 class="title">{ "About Us" }</h1>
                <div class="content">
                    <p>{ "Thoth is an open metadata management and dissemination platform. As an organisation, we are focused on:" }</p>
                    <ul>
                        <li>{ "creation, curation, and dissemination of high-quality metadata records which are essential for the advancement of public knowledge;" }</li>
                        <li>{ "promoting openness in scholarly communications such as open access, open licensing, FLOSS, open data, open metadata, and open standards and protocols;" }</li>
                        <li>{ "embracing infrastructural and platform diversity as an inherent component of a flourishing scholarly communications landscape;" }</li>
                        <li>{ "providing high-quality solutions and services for metadata creation, management, dissemination, archiving and preservation." }</li>
                    </ul>
                    <p>{ "Thoth has been developed by Javier Arias in the context of the Community-led Open Publication Infrastructures for Monographs (COPIM) project funded by UKRI and the Arcadia Fund." }</p>
                    <p>{ "Thoth is incorporated as a Community Interest Company in the UK, no. 14549556." }</p>
                    <p>{ "Address: 40 Devonshire Road, Cambridge, United Kingdom, CB1 2BL" }</p>
                    <p class="title is-5">{ "Organisation" }</p>
                    <ul>
                        <li>{ "Joe Deville (Director)" }</li>
                        <li>{ "Rupert Gatti (Director)" }</li>
                        <li>{ "Vincent W.J. van Gerven Oei (Director)" }</li>
                        <li>{ "Javier Arias (CTO)" }</li>
                        <li>{ "Ross Higman (Software Engineer)" }</li>
                    </ul>
                    <p>{ "Support us through the " }<a href={ "https://www.openbookcollective.org/packages/20/info/" }>{ "Open Book Collective" }</a>{ "." }</p>
                    <p>{ "Contact us at " }<a href={ "mailto:info@thoth.pub" }>{ "info@thoth.pub" }</a>{ "." }</p>
                </div>
            </div>
        }
    }
}
