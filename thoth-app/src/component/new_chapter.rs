#![allow(clippy::unnecessary_operation)]

use thoth_api::model::work::WorkType;
use thoth_api::model::work::WorkWithRelations;
use thoth_api::model::work_relation::RelationType;
use thoth_api::model::work_relation::WorkRelationWithRelatedWork;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
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

use super::ToElementValue;

pub struct NewChapterComponent {
    new_chapter_title: String,
    new_relation: WorkRelationWithRelatedWork,
    show_add_form: bool,
    push_work: PushCreateWork,
    push_relation: PushCreateWorkRelation,
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

    fn create(_ctx: &Context<Self>) -> Self {
        let new_relation: WorkRelationWithRelatedWork = Default::default();
        let new_chapter_title = Default::default();
        let show_add_form = false;
        let push_relation = Default::default();
        let push_work = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        NewChapterComponent {
            new_relation,
            new_chapter_title,
            show_add_form,
            push_relation,
            push_work,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                if value {
                    // On opening form, set chapter number to one higher than the current maximum
                    // (may not be the most appropriate value if user has left gaps in numbering)
                    let max_chapter_num = ctx
                        .props()
                        .relations
                        .clone()
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|r| r.relation_type == RelationType::HasChild)
                        .max_by_key(|r| r.relation_ordinal)
                        .map(|r| r.relation_ordinal)
                        .unwrap_or(0);
                    self.new_relation.relation_ordinal = max_chapter_num + 1;
                }
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
                            let relation = r.clone();
                            let mut relations: Vec<WorkRelationWithRelatedWork> =
                                ctx.props().relations.clone().unwrap_or_default();
                            relations.push(relation);
                            ctx.props().update_relations.emit(Some(relations));
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                format!(
                                    "Created new work with title {}, but failed to add it to Related Works list",
                                    self.new_chapter_title
                                ),
                                NotificationStatus::Warning,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            format!(
                                "Created new work with title {}, but failed to add it to Related Works list: {}",
                                self.new_chapter_title,
                                ThothError::from(err),
                            ),
                            NotificationStatus::Warning,
                        )));
                        false
                    }
                }
            }
            Msg::CreateWorkRelation(new_chapter_id) => {
                let body = CreateWorkRelationRequestBody {
                    variables: CreateVariables {
                        relator_work_id: ctx.props().work.work_id,
                        related_work_id: new_chapter_id,
                        relation_type: RelationType::HasChild,
                        relation_ordinal: self.new_relation.relation_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateWorkRelationRequest { body };
                self.push_relation = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_relation.fetch(Msg::SetRelationPushState));
                ctx.link()
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
                            // New Book Chapter successfully created.
                            // Now add a new Work Relation linking it to the parent.
                            ctx.link().send_message(Msg::CreateWorkRelation(w.work_id));
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to create new chapter".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateWork => {
                // First, create a new Book Chapter with values inherited from current Work.
                let body = CreateWorkRequestBody {
                    variables: Variables {
                        work_type: WorkType::BookChapter,
                        work_status: ctx.props().work.work_status,
                        full_title: self.new_chapter_title.clone(),
                        title: self.new_chapter_title.clone(),
                        publication_date: ctx.props().work.publication_date,
                        place: ctx.props().work.place.clone(),
                        license: ctx.props().work.license.clone(),
                        imprint_id: ctx.props().work.imprint.imprint_id,
                        // All others can be set to None/blank/default
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let request = CreateWorkRequest { body };
                self.push_work = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_work.fetch(Msg::SetWorkPushState));
                ctx.link()
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <>
                <div class="control" style="margin-bottom: 1em">
                    <a
                        class="button is-success"
                        onclick={ ctx.link().callback(move |_| Msg::ToggleAddFormDisplay(true)) }
                    >
                        { "Add new Chapter" }
                    </a>
                </div>
                <div class={ self.add_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Chapter" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="chapter-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateWork
                            }) }
                            >
                                <FormNumberInput
                                    label = "Chapter Number"
                                    value={ self.new_relation.relation_ordinal }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeOrdinal(e.to_value())) }
                                    required = true
                                    min={ "1".to_string() }
                                />
                                <FormTextInput
                                    label = "Chapter Title"
                                    value={ self.new_chapter_title.clone() }
                                    oninput={ ctx.link().callback(|e: InputEvent| Msg::ChangeTitle(e.to_value())) }
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
                                        { &ctx.props().work.work_status }
                                    </div>
                                </div>
                                <div class="field">
                                    <label class="label">{ "Chapter Imprint" }</label>
                                    <div class="control is-expanded">
                                        { &ctx.props().work.imprint.imprint_name }
                                    </div>
                                </div>
                                {
                                    if let Some(date) = &ctx.props().work.publication_date {
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
                                    if let Some(place) = &ctx.props().work.place {
                                        html! {
                                            <div class="field">
                                                <label class="label">{ "Chapter Place of Publication" }</label>
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
                                    if let Some(license) = &ctx.props().work.license {
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
                                onclick={ &close_modal }
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
