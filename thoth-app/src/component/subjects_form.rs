use std::str::FromStr;
use thoth_api::subject::model::Subject;
use thoth_api::subject::model::SubjectType;
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
use crate::component::utils::FormSubjectTypeSelect;
use crate::component::utils::FormTextInput;
use crate::models::subject::create_subject_mutation::CreateSubjectRequest;
use crate::models::subject::create_subject_mutation::CreateSubjectRequestBody;
use crate::models::subject::create_subject_mutation::PushActionCreateSubject;
use crate::models::subject::create_subject_mutation::PushCreateSubject;
use crate::models::subject::create_subject_mutation::Variables;
use crate::models::subject::delete_subject_mutation::DeleteSubjectRequest;
use crate::models::subject::delete_subject_mutation::DeleteSubjectRequestBody;
use crate::models::subject::delete_subject_mutation::PushActionDeleteSubject;
use crate::models::subject::delete_subject_mutation::PushDeleteSubject;
use crate::models::subject::delete_subject_mutation::Variables as DeleteVariables;
use crate::models::subject::subject_types_query::FetchActionSubjectTypes;
use crate::models::subject::subject_types_query::FetchSubjectTypes;
use crate::models::subject::SubjectTypeValues;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_SUBJECTS;
use crate::string::REMOVE_BUTTON;

pub struct SubjectsFormComponent {
    props: Props,
    data: SubjectsFormData,
    new_subject: Subject,
    show_add_form: bool,
    fetch_subject_types: FetchSubjectTypes,
    push_subject: PushCreateSubject,
    delete_subject: PushDeleteSubject,
    link: ComponentLink<Self>,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct SubjectsFormData {
    subject_types: Vec<SubjectTypeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetSubjectTypesFetchState(FetchActionSubjectTypes),
    GetSubjectTypes,
    SetSubjectPushState(PushActionCreateSubject),
    CreateSubject,
    SetSubjectDeleteState(PushActionDeleteSubject),
    DeleteSubject(Uuid),
    ChangeSubjectType(SubjectType),
    ChangeCode(String),
    ChangeOrdinal(String),
    DoNothing,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub subjects: Option<Vec<Subject>>,
    pub work_id: Uuid,
    pub update_subjects: Callback<Option<Vec<Subject>>>,
}

impl Component for SubjectsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data: SubjectsFormData = Default::default();
        let show_add_form = false;
        let new_subject: Subject = Default::default();
        let push_subject = Default::default();
        let delete_subject = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        link.send_message(Msg::GetSubjectTypes);

        SubjectsFormComponent {
            props,
            data,
            new_subject,
            show_add_form,
            fetch_subject_types: Default::default(),
            push_subject,
            delete_subject,
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
            Msg::SetSubjectTypesFetchState(fetch_state) => {
                self.fetch_subject_types.apply(fetch_state);
                self.data.subject_types = match self.fetch_subject_types.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.subject_types.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetSubjectTypes => {
                self.link.send_future(
                    self.fetch_subject_types
                        .fetch(Msg::SetSubjectTypesFetchState),
                );
                self.link
                    .send_message(Msg::SetSubjectTypesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetSubjectPushState(fetch_state) => {
                self.push_subject.apply(fetch_state);
                match self.push_subject.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_subject {
                        Some(p) => {
                            let subject = p.clone();
                            let mut subjects: Vec<Subject> =
                                self.props.subjects.clone().unwrap_or_default();
                            subjects.push(subject);
                            self.props.update_subjects.emit(Some(subjects));
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            self.link.send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.link.send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateSubject => {
                let body = CreateSubjectRequestBody {
                    variables: Variables {
                        work_id: self.props.work_id,
                        subject_type: self.new_subject.subject_type.clone(),
                        subject_code: self.new_subject.subject_code.clone(),
                        subject_ordinal: self.new_subject.subject_ordinal,
                    },
                    ..Default::default()
                };
                let request = CreateSubjectRequest { body };
                self.push_subject = Fetch::new(request);
                self.link
                    .send_future(self.push_subject.fetch(Msg::SetSubjectPushState));
                self.link
                    .send_message(Msg::SetSubjectPushState(FetchAction::Fetching));
                false
            }
            Msg::SetSubjectDeleteState(fetch_state) => {
                self.delete_subject.apply(fetch_state);
                match self.delete_subject.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_subject {
                        Some(subject) => {
                            let to_keep: Vec<Subject> = self
                                .props
                                .subjects
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|p| p.subject_id != subject.subject_id)
                                .collect();
                            self.props.update_subjects.emit(Some(to_keep));
                            true
                        }
                        None => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            err.to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::DeleteSubject(subject_id) => {
                let body = DeleteSubjectRequestBody {
                    variables: DeleteVariables { subject_id },
                    ..Default::default()
                };
                let request = DeleteSubjectRequest { body };
                self.delete_subject = Fetch::new(request);
                self.link
                    .send_future(self.delete_subject.fetch(Msg::SetSubjectDeleteState));
                self.link
                    .send_message(Msg::SetSubjectDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeSubjectType(val) => self.new_subject.subject_type.neq_assign(val),
            Msg::ChangeCode(code) => self.new_subject.subject_code.neq_assign(code),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap_or(0);
                self.new_subject.subject_ordinal.neq_assign(ordinal);
                false // otherwise we re-render the component and reset the value
            }
            Msg::DoNothing => false, // callbacks need to return a message
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let subjects = self.props.subjects.clone().unwrap_or_default();
        let open_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(true)
        });
        let close_modal = self.link.callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Subjects" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick=open_modal
                    >
                        { "Add Subject" }
                    </button>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Subject" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick=&close_modal
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form onsubmit=self.link.callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::DoNothing
                            })
                            >
                                <FormSubjectTypeSelect
                                    label = "Subject Type"
                                    value=self.new_subject.subject_type.clone()
                                    data=self.data.subject_types.clone()
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeSubjectType(
                                                SubjectType::from_str(&value).unwrap()
                                            )
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormTextInput
                                    label = "Subject Code"
                                    value=self.new_subject.subject_code.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeCode(e.value))
                                />
                                <FormNumberInput
                                    label = "Subject Ordinal"
                                    value=self.new_subject.subject_ordinal
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeOrdinal(e.value))
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::CreateSubject
                                })
                            >
                                { "Add Subject" }
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
                {
                    if !subjects.is_empty() {
                        html!{{for subjects.iter().map(|p| self.render_subject(p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_SUBJECTS }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl SubjectsFormComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_subject(&self, s: &Subject) -> Html {
        let subject_id = s.subject_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-tag" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Subject Type" }</label>
                        <div class="control is-expanded">
                            {&s.subject_type}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Subject Code" }</label>
                        <div class="control is-expanded">
                            {&s.subject_code.clone()}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Subject Ordinal" }</label>
                        <div class="control is-expanded">
                            {&s.subject_ordinal.clone()}
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::DeleteSubject(subject_id))
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
