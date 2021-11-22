use uuid::Uuid;
use yew_router::prelude::*;
use yew_router::switch::Permissive;

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AppRoute {
    #[to = "/login"]
    Login,
    #[to = "/error"]
    Error(Permissive<String>),
    #[to = "/admin{*:rest}"]
    Admin(AdminRoute),
    #[to = "/"]
    Home,
}

#[derive(Switch, Debug, Clone, PartialEq)]
pub enum AdminRoute {
    #[to = "/dashboard"]
    Dashboard,
    #[to = "/works"]
    Works,
    #[to = "/books"]
    Books,
    #[to = "/chapters"]
    Chapters,
    #[to = "/work/{id}"]
    Work(Uuid),
    #[to = "/work"]
    NewWork,
    #[to = "/publishers"]
    Publishers,
    #[to = "/publisher/{id}"]
    Publisher(Uuid),
    #[to = "/publisher"]
    NewPublisher,
    #[to = "/funders"]
    Funders,
    #[to = "/funder/{id}"]
    Funder(Uuid),
    #[to = "/funder"]
    NewFunder,
    #[to = "/imprints"]
    Imprints,
    #[to = "/imprint/{id}"]
    Imprint(Uuid),
    #[to = "/imprint"]
    NewImprint,
    #[to = "/contributors"]
    Contributors,
    #[to = "/contributor/{id}"]
    Contributor(Uuid),
    #[to = "/contributor"]
    NewContributor,
    #[to = "/serieses"]
    Serieses,
    #[to = "/series/{id}"]
    Series(Uuid),
    #[to = "/series"]
    NewSeries,
    #[to = "/publications"]
    Publications,
    #[to = "/publication/{id}"]
    Publication(Uuid),
    #[to = "/publication"]
    NewPublication,
    #[to = ""]
    Admin,
}
