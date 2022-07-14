use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use thoth_api::model::work_relation::RelationType;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormNumberInput;
use crate::component::utils::FormTextInput;
use crate::models::work::create_work_mutation::CreateWorkRequest;
use crate::models::work::create_work_mutation::CreateWorkRequestBody;
use crate::models::work::create_work_mutation::PushActionCreateWork;
use crate::models::work::create_work_mutation::PushCreateWork;
use crate::models::work::create_work_mutation::Variables;
use crate::models::work_relation::create_work_relation_mutation::CreateWorkRelationRequest;
use crate::models::work_relation::create_work_relation_mutation::CreateWorkRelationRequestBody;
use crate::models::work_relation::create_work_relation_mutation::PushActionCreateWorkRelation;
use crate::models::work_relation::create_work_relation_mutation::PushCreateWorkRelation;
use crate::models::work_relation::create_work_relation_mutation::Variables as CreateVariables;
use crate::string::CANCEL_BUTTON;
use crate::string::NEW_CHAPTER_INFO;

pub struct NewChapterComponent {
    props: Props,
    new_chapter_title: String,
    new_relation: WorkRelationWithRelatedWork,
    show_add_form: bool,
    push_work: PushCreateWork,
    push_relation: PushCreateWorkRelation,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[allow(clippy::large_enum_variant)]
pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetRelationPushState(PushActionCreateWorkRelation),
    CreateWorkRelation(Uuid),
    SetWorkPushState(PushActionCreateWork),
    CreateWork,
    ChangeOrdinal(String),
    ChangeTitle(String),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub work: WorkWithRelations,
    pub relations: Option<Vec<WorkRelationWithRelatedWork>>,
    pub update_relations: Callback<Option<Vec<WorkRelationWithRelatedWork>>>,
}

impl Component for NewChapterComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let new_relation: WorkRelationWithRelatedWork = Default::default();
        let new_chapter_title = Default::default();
        let show_add_form = false;
        let push_relation = Default::default();
        let push_work = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        NewChapterComponent {
            props,
            new_relation,
            new_chapter_title,
            show_add_form,
            push_relation,
            push_work,
            link,
            notification_bus,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetRelationPushState(fetch_state) => {
                self.push_relation.apply(fetch_state);
                match self.push_relation.clone().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_work_relation {
                        Some(r) => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!(
                                    "Added new work to Related Works list as Chapter {}",
                                    self.new_relation.relation_ordinal
                                ),
                                NotificationStatus::Success,
                            )));
                            let relation = r.clone();
                            let mut relations: Vec<WorkRelationWithRelatedWork> =
                                self.props.relations.clone().unwrap_or_default();
                            relations.push(relation);
                            self.props.update_relations.emit(Some(relations));
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save relation between current work and new work"
                                    .to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link.send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            format!("Failed to add new work to Related Works list: {}", err),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateWorkRelation(new_chapter_id) => {
                let body = CreateWorkRelationRequestBody {
                    variables: CreateVariables {
                        relator_work_id: self.props.work.work_id,
                        related_work_id: new_chapter_id,
                        relation_type: RelationType::HasChild,
                        relation_ordinal: self.new_relation.relation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateWorkRelationRequest { body };
                self.push_relation = Fetch::new(request);
                self.link
                    .send_future(self.push_relation.fetch(Msg::SetRelationPushState));
                self.link
                    .send_message(Msg::SetRelationPushState(FetchAction::Fetching));
                false
            }
            Msg::SetWorkPushState(fetch_state) => {
                self.push_work.apply(fetch_state);
                match self.push_work.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_work {
                        Some(w) => {
                            self.link.send_message(Msg::CreateWorkRelation(w.work_id));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!("Created new work with title {}", w.title),
                                NotificationStatus::Success,
                            )));
                            true
                        }
                        None => {
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save new work".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link.send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            format!("Failed to create new work: {}", err),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateWork => {
                let body = CreateWorkRequestBody {
                    variables: Variables {
                        work_type: WorkType::BookChapter,
                        work_status: self.props.work.work_status.clone(),
                        full_title: self.new_chapter_title.clone(),
                        title: self.new_chapter_title.clone(),
                        publication_date: self.props.work.publication_date.clone(),
                        place: self.props.work.place.clone(),
                        license: self.props.work.license.clone(),
                        // Copyright Holder is a mandatory field so must be inherited
                        // even though it may not be the same for edited books.
                        copyright_holder: self.props.work.copyright_holder.clone(),
                        imprint_id: self.props.work.imprint.imprint_id,
                        // All others can be set to None/blank/default
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = CreateWorkRequest { body };
                self.push_work = Fetch::new(request);
                self.link
                    .send_future(self.push_work.fetch(Msg::SetWorkPushState));
                self.link
                    .send_message(Msg::SetWorkPushState(FetchAction::Fetching));
                false
            }
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.new_relation.relation_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
            Msg::ChangeTitle(title) => self.new_chapter_title.neq_assign(title.trim().to_owned()),
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <>
                <div class="control">
                    <a
                        class="button is-success"
                        onclick=self.link.callback(move |_| Msg::ToggleAddFormDisplay(true))
                    >
                        { "Add new Chapter" }
                    </a>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Chapter" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="chapter-form" onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateWork
                            })
                            >
                                <FormNumberInput
                                    label = "Chapter Number"
                                    value=self.new_relation.relation_ordinal
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeOrdinal(e.value))
                                    required = true
                                    min = "1".to_string()
                                />
                                <FormTextInput
                                    label = "Chapter Title"
                                    value=self.new_chapter_title.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeTitle(e.value))
                                    required = true
                                />
                                <article class="message is-info">
                                    <div class="message-body">
                                        { NEW_CHAPTER_INFO }
                                    </div>
                                </article>
                                <div class="field">
                                    <label class="label">{ "Relation between Work and Chapter" }</label>
                                    <div class="control is-expanded">
                                        { RelationType::HasChild }
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{ "Chapter Work Type" }</label>
                                    <div class="control is-expanded">
                                        { WorkType::BookChapter }
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{ "Chapter Work Status" }</label>
                                    <div class="control is-expanded">
                                        { &self.props.work.work_status }
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{ "Chapter Imprint" }</label>
                                    <div class="control is-expanded">
                                        { &self.props.work.imprint.imprint_name }
                                    </div>
                                </div>
                                {
                                    if let Some(date) = &self.props.work.publication_date {
                                        html! {
                                            <div class="field">
                                                <label class="label">{ "Chapter Publication Date" }</label>
                                                <div class="control is-expanded">
                                                    { date }
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        html!{}
                                    }
                                }
                                {
                                    if let Some(place) = &self.props.work.place {
                                        html! {
                                            <div class="field">
                                                <label class="label">{ "Chapter Place of Publicatio" }</label>
                                                <div class="control is-expanded">
                                                    { place }
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        html!{}
                                    }
                                }
                                {
                                    if let Some(license) = &self.props.work.license {
                                        html! {
                                            <div class="field">
                                                <label class="label">{ "Chapter License" }</label>
                                                <div class="control is-expanded">
                                                    { license }
                                                </div>
                                            </div>
                                        }
                                    } else {
                                        html!{}
                                    }
                                }
                                <div class="field">
                                    <label class="label">{ "Chapter Copyright Holder" }</label>
                                    <div class="control is-expanded">
                                        { &self.props.work.copyright_holder }
                                    </div>
                                </div>
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="chapter-form"
                            >
                                { "Add Chapter" }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
            </>
        }
    }
}

impl NewChapterComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }
}
