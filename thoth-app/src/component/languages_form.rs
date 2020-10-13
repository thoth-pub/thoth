use std::str::FromStr;
use thoth_api::models::language::LanguageCode;
use thoth_api::models::language::LanguageRelation;
use yew::html;
use yew::prelude::*;
use yew::ComponentLink;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::future::LinkFuture;
use yewtil::NeqAssign;

use crate::models::language::language_codes_query::FetchActionLanguageCodes;
use crate::models::language::language_codes_query::FetchLanguageCodes;
use crate::models::language::language_relations_query::FetchActionLanguageRelations;
use crate::models::language::language_relations_query::FetchLanguageRelations;
use crate::models::language::Language;
use crate::models::language::LanguageCodeValues;
use crate::models::language::LanguageRelationValues;
use crate::component::utils::FormLanguageCodeSelect;
use crate::component::utils::FormLanguageRelationSelect;
use crate::component::utils::FormBooleanSelect;
use crate::string::YES;
use crate::string::NO;
use crate::string::EMPTY_LANGUAGES;
use crate::string::REMOVE_BUTTON;

pub struct LanguagesFormComponent {
    props: Props,
    data: LanguagesFormData,
    new_language: Language,
    show_add_form: bool,
    fetch_language_codes: FetchLanguageCodes,
    fetch_language_relations: FetchLanguageRelations,
    link: ComponentLink<Self>,
}

struct LanguagesFormData {
    language_codes: Vec<LanguageCodeValues>,
    language_relations: Vec<LanguageRelationValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetLanguageCodesFetchState(FetchActionLanguageCodes),
    GetLanguageCodes,
    SetLanguageRelationsFetchState(FetchActionLanguageRelations),
    GetLanguageRelations,
    ChangeLanguageCode(LanguageCode),
    ChangeLanguageRelation(LanguageRelation),
    ChangeMainLanguage(bool),
    AddLanguage,
    RemoveLanguage(String),
    DoNothing,
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub languages: Option<Vec<Language>>,
    pub work_id: String,
    pub update_languages: Callback<Option<Vec<Language>>>,
}

impl Component for LanguagesFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let data = LanguagesFormData {
            language_codes: vec![],
            language_relations: vec![],
        };
        let show_add_form = false;
        let new_language: Language = Default::default();

        link.send_message(Msg::GetLanguageCodes);
        link.send_message(Msg::GetLanguageRelations);

        LanguagesFormComponent {
            props,
            data,
            new_language,
            show_add_form,
            fetch_language_codes: Default::default(),
            fetch_language_relations: Default::default(),
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetLanguageCodesFetchState(fetch_state) => {
                self.fetch_language_codes.apply(fetch_state);
                self.data.language_codes = match self.fetch_language_codes.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.language_codes.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetLanguageCodes => {
                self.link.send_future(
                    self.fetch_language_codes.fetch(Msg::SetLanguageCodesFetchState),
                );
                self.link
                    .send_message(Msg::SetLanguageCodesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetLanguageRelationsFetchState(fetch_state) => {
                self.fetch_language_relations.apply(fetch_state);
                self.data.language_relations = match self.fetch_language_relations.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.language_relations.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetLanguageRelations => {
                self.link.send_future(
                    self.fetch_language_relations
                        .fetch(Msg::SetLanguageRelationsFetchState),
                );
                self.link
                    .send_message(Msg::SetLanguageRelationsFetchState(FetchAction::Fetching));
                false
            }
            Msg::ChangeLanguageRelation(val) => self.new_language.language_relation.neq_assign(val),
            Msg::ChangeLanguageCode(code) => self.new_language.language_code.neq_assign(code),
            Msg::ChangeMainLanguage(val) => self.new_language.main_language.neq_assign(val),
            Msg::AddLanguage => {
                let language = self.new_language.clone();
                let mut languages: Vec<Language> = self.props.languages.clone().unwrap_or_default();
                let language = Language {
                    language_id: language.language_id,
                    work_id: self.props.work_id.clone(),
                    language_relation: language.language_relation,
                    language_code: language.language_code,
                    main_language: language.main_language,
                };
                languages.push(language);
                self.new_language = Default::default();
                self.props.update_languages.emit(Some(languages));
                self.link.send_message(Msg::ToggleAddFormDisplay(false));
                true
            }
            Msg::RemoveLanguage(language_id) => {
                let to_keep: Vec<Language> = self
                    .props
                    .languages
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|i| i.language_id != language_id)
                    .collect();
                self.props.update_languages.emit(Some(to_keep));
                true
            }
            Msg::DoNothing => false,  // callbacks need to return a message
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        let languages = self.props.languages.clone().unwrap_or_default();
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
                    { "Languages" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick=open_modal
                    >
                        { "Add Language" }
                    </button>
                </div>
                <div class=self.add_form_status()>
                    <div class="modal-background" onclick=&close_modal></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Language" }</p>
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
                                <FormLanguageCodeSelect
                                    label = "Language Code"
                                    value=&self.new_language.language_code
                                    data=&self.data.language_codes
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeLanguageCode(
                                                LanguageCode::from_str(&value).unwrap()
                                            )
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormLanguageRelationSelect
                                    label = "Language Relation"
                                    value=&self.new_language.language_relation
                                    data=&self.data.language_relations
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            Msg::ChangeLanguageRelation(
                                                LanguageRelation::from_str(&value).unwrap()
                                            )
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                                <FormBooleanSelect
                                    label = "Main"
                                    value=&self.new_language.main_language
                                    onchange=self.link.callback(|event| match event {
                                        ChangeData::Select(elem) => {
                                            let value = elem.value();
                                            let boolean = value == "true";
                                            Msg::ChangeMainLanguage(boolean)
                                        }
                                        _ => unreachable!(),
                                    })
                                    required = true
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                onclick=self.link.callback(|e: MouseEvent| {
                                    e.prevent_default();
                                    Msg::AddLanguage
                                })
                            >
                                { "Add Language" }
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
                    if languages.len() > 0 {
                        html!{{for languages.iter().map(|p| self.render_language(p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_LANGUAGES }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl LanguagesFormComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_language(&self, l: &Language) -> Html {
        // there's probably a better way to do this. We basically need to copy 3 instances
        // of contributor_id and take ownership of them so they can be passed on to
        // the callback functions
        let language_id = l.language_id.clone();
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-language" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Language Code" }</label>
                        <div class="control is-expanded">
                            {&l.language_code}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Language Relation" }</label>
                        <div class="control is-expanded">
                            {&l.language_relation}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Main" }</label>
                        <div class="control is-expanded">
                            {
                                match &l.main_language {
                                    true => { YES },
                                    false => { NO }
                                }
                            }
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick=self.link.callback(move |_| Msg::RemoveLanguage(language_id.clone()))
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
