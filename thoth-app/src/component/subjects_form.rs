use std::str::FromStr;
use thoth_api::subject::model::SubjectType;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::component::utils::FormNumberInput;
use crate::component::utils::FormSubjectTypeSelect;
use crate::component::utils::FormTextInput;
use crate::models::subject::subject_types_query::FetchActionSubjectTypes;
use crate::models::subject::subject_types_query::FetchSubjectTypes;
use crate::models::subject::Subject;
use crate::models::subject::SubjectTypeValues;
use crate::string::EMPTY_SUBJECTS;
use crate::string::REMOVE_BUTTON;

pub struct SubjectsFormComponent {
    props: Props,
    data: SubjectsFormData,
    new_subject: Subject,
    show_add_form: bool,
    fetch_subject_types: FetchSubjectTypes,
    link: ComponentLink<Self>,
}

struct SubjectsFormData {
    subject_types: Vec<SubjectTypeValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetSubjectTypesFetchState(FetchActionSubjectTypes),
    GetSubjectTypes,
    ChangeSubjectType(SubjectType),
    ChangeCode(String),
    ChangeOrdinal(String),
    AddSubject,
    RemoveSubject(String),
    DoNothing,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub subjects: Option<Vec<Subject>>,
    pub work_id: String,
    pub update_subjects: Callback<Option<Vec<Subject>>>,
}

impl Component for SubjectsFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data = SubjectsFormData {
            subject_types: vec![],
        };
        let show_add_form = false;
        let new_subject: Subject = Default::default();

        link.send_message(Msg::GetSubjectTypes);

        SubjectsFormComponent {
            props,
            data,
            new_subject,
            show_add_form,
            fetch_subject_types: Default::default(),
            link,
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
            Msg::ChangeSubjectType(val) => self.new_subject.subject_type.neq_assign(val),
            Msg::ChangeCode(code) => self.new_subject.subject_code.neq_assign(code),
            Msg::ChangeOrdinal(ordinal) => {
                let ordinal = ordinal.parse::<i32>().unwrap();
                self.new_subject.subject_ordinal.neq_assign(ordinal)
            }
            Msg::AddSubject => {
                let subject = self.new_subject.clone();
                let mut subjects: Vec<Subject> = self.props.subjects.clone().unwrap_or_default();
                let subject = Subject {
                    subject_id: subject.subject_id,
                    work_id: self.props.work_id.clone(),
                    subject_type: subject.subject_type,
                    subject_code: subject.subject_code,
                    subject_ordinal: subject.subject_ordinal,
                };
                subjects.push(subject);
                self.new_subject = Default::default();
                self.props.update_subjects.emit(Some(subjects));
                self.link.send_message(Msg::ToggleAddFormDisplay(false));
                true
            }
            Msg::RemoveSubject(subject_id) => {
                let to_keep: Vec<Subject> = self
                    .props
                    .subjects
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|i| i.subject_id != subject_id)
                    .collect();
                self.props.update_subjects.emit(Some(to_keep));
                true
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
                                    value=&self.new_subject.subject_type
                                    data=&self.data.subject_types
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
                                    value=&self.new_subject.subject_code.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeCode(e.value))
                                />
                                <FormNumberInput
                                    label = "Subject Ordinal"
                                    value=&self.new_subject.subject_ordinal.clone()
                                    oninput=self.link.callback(|e: InputData| Msg::ChangeOrdinal(e.value))
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::AddSubject
                                })
                            >
                                { "Add Subject" }
                            </button>
                            <button
                                class="button"
                                onclick=&close_modal
                            >
                                { "Cancel" }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if subjects.len() > 0 {
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
        // there's probably a better way to do this. We basically need to copy 3 instances
        // of contributor_id and take ownership of them so they can be passed on to
        // the callback functions
        let subject_id = s.subject_id.clone();
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
                                onclick=self.link.callback(move |_| Msg::RemoveSubject(subject_id.clone()))
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
