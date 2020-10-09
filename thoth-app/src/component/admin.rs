use yew::html;
use yew::prelude::*;
use yew::ComponentLink;

use crate::component::contributors::ContributorsComponent;
use crate::component::dashboard::DashboardComponent;
use crate::component::imprints::ImprintsComponent;
use crate::component::menu::MenuComponent;
use crate::component::contributor::ContributorComponent;
use crate::component::publications::PublicationsComponent;
use crate::component::publishers::PublishersComponent;
use crate::component::serieses::SeriesesComponent;
use crate::component::work::WorkComponent;
use crate::component::works::WorksComponent;
use crate::route::AdminRoute;

pub struct AdminComponent {
    props: Props,
}

pub enum Msg {}

#[derive(Clone, Properties)]
pub struct Props {
    pub route: AdminRoute,
}

impl Component for AdminComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        AdminComponent { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="columns">
                <div class="column">
                    <div class="container">
                        <MenuComponent route = &self.props.route />
                    </div>
                </div>
                <div class="column is-four-fifths">
                    <div class="container">
                    {
                        match &self.props.route {
                            AdminRoute::Dashboard => html!{<DashboardComponent/>},
                            AdminRoute::Works => html!{<WorksComponent/>},
                            AdminRoute::Work(id) => html!{<WorkComponent work_id = id />},
                            AdminRoute::Publishers => html!{<PublishersComponent/>},
                            AdminRoute::Publisher(id) => html!{{ id }},
                            AdminRoute::Imprints => html!{<ImprintsComponent/>},
                            AdminRoute::Imprint(id) => html!{{ id }},
                            AdminRoute::Publications => html!{<PublicationsComponent/>},
                            AdminRoute::Publication(id) => html!{{ id }},
                            AdminRoute::Contributors => html!{<ContributorsComponent/>},
                            AdminRoute::Contributor(id) => html!{<ContributorComponent contributor_id = id />},
                            AdminRoute::Serieses => html!{<SeriesesComponent/>},
                            AdminRoute::Series(id) => html!{{ id }},
                            AdminRoute::Admin => html!{<DashboardComponent/>},
                        }
                    }
                    </div>
                </div>
            </div>
        }
    }
}
