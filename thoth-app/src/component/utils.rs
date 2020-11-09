use thoth_api::contribution::model::ContributionType;
use thoth_api::language::model::LanguageCode;
use thoth_api::language::model::LanguageRelation;
use thoth_api::publication::model::PublicationType;
use thoth_api::series::model::SeriesType;
use thoth_api::subject::model::SubjectType;
use thoth_api::work::model::WorkStatus;
use thoth_api::work::model::WorkType;
use yew::html;
use yew::virtual_dom::VNode;
use yew::Callback;
use yew::ChangeData;
use yew::FocusEvent;
use yew::InputData;
use yew::MouseEvent;
use yew::Properties;
use yewtil::Pure;
use yewtil::PureComponent;

use crate::models::contribution::ContributionTypeValues;
use crate::models::imprint::Imprint;
use crate::models::language::LanguageCodeValues;
use crate::models::language::LanguageRelationValues;
use crate::models::publication::PublicationTypeValues;
use crate::models::publisher::Publisher;
use crate::models::series::SeriesTypeValues;
use crate::models::subject::SubjectTypeValues;
use crate::models::work::WorkStatusValues;
use crate::models::work::WorkTypeValues;
use crate::string::NO;
use crate::string::RELOAD_BUTTON;
use crate::string::YES;

pub type FormInput = Pure<PureInput>;
pub type FormTextarea = Pure<PureTextarea>;
pub type FormTextInput = Pure<PureTextInput>;
pub type FormUrlInput = Pure<PureUrlInput>;
pub type FormDateInput = Pure<PureDateInput>;
pub type FormNumberInput = Pure<PureNumberInput>;
pub type FormWorkTypeSelect = Pure<PureWorkTypeSelect>;
pub type FormWorkStatusSelect = Pure<PureWorkStatusSelect>;
pub type FormContributionTypeSelect = Pure<PureContributionTypeSelect>;
pub type FormPublicationTypeSelect = Pure<PurePublicationTypeSelect>;
pub type FormSeriesTypeSelect = Pure<PureSeriesTypeSelect>;
pub type FormSubjectTypeSelect = Pure<PureSubjectTypeSelect>;
pub type FormLanguageCodeSelect = Pure<PureLanguageCodeSelect>;
pub type FormLanguageRelationSelect = Pure<PureLanguageRelationSelect>;
pub type FormBooleanSelect = Pure<PureBooleanSelect>;
pub type FormImprintSelect = Pure<PureImprintSelect>;
pub type FormPublisherSelect = Pure<PurePublisherSelect>;
pub type Loader = Pure<PureLoader>;
pub type Reloader = Pure<PureReloader>;

#[derive(Clone, PartialEq, Properties)]
pub struct PureInput {
    pub label: String,
    pub value: String,
    pub input_type: String,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureTextarea {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureTextInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureUrlInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureDateInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureNumberInput {
    pub label: String,
    pub value: Option<i32>,
    #[prop_or_default]
    pub oninput: Callback<InputData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureWorkTypeSelect {
    pub label: String,
    pub data: Vec<WorkTypeValues>,
    pub value: WorkType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureWorkStatusSelect {
    pub label: String,
    pub data: Vec<WorkStatusValues>,
    pub value: WorkStatus,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureContributionTypeSelect {
    pub label: String,
    pub data: Vec<ContributionTypeValues>,
    pub value: ContributionType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PurePublicationTypeSelect {
    pub label: String,
    pub data: Vec<PublicationTypeValues>,
    pub value: PublicationType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureSubjectTypeSelect {
    pub label: String,
    pub data: Vec<SubjectTypeValues>,
    pub value: SubjectType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureSeriesTypeSelect {
    pub label: String,
    pub data: Vec<SeriesTypeValues>,
    pub value: SeriesType,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLanguageCodeSelect {
    pub label: String,
    pub data: Vec<LanguageCodeValues>,
    pub value: LanguageCode,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLanguageRelationSelect {
    pub label: String,
    pub data: Vec<LanguageRelationValues>,
    pub value: LanguageRelation,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureBooleanSelect {
    pub label: String,
    pub value: bool,
    pub onchange: Callback<ChangeData>,
    #[prop_or_default]
    pub onblur: Callback<FocusEvent>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureImprintSelect {
    pub label: String,
    pub data: Vec<Imprint>,
    pub value: Option<String>,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PurePublisherSelect {
    pub label: String,
    pub data: Vec<Publisher>,
    pub value: Option<String>,
    pub onchange: Callback<ChangeData>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureLoader {}

#[derive(Clone, PartialEq, Properties)]
pub struct PureReloader {
    pub onclick: Callback<MouseEvent>,
}

impl PureComponent for PureInput {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <input
                        class="input"
                        type={ &self.input_type }
                        placeholder={ &self.label }
                        value={ &self.value }
                        oninput=&self.oninput
                        onblur=&self.onblur
                        required={ self.required }
                    />
                </div>
            </div>
        }
    }
}

impl PureComponent for PureTextarea {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <textarea
                        class="textarea"
                        placeholder=&self.label
                        oninput=&self.oninput
                        required={ self.required }
                    >
                        {&self.value.clone().unwrap_or_else(|| "".to_string())}
                    </textarea>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureTextInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=&self.label
                value=&self.value.clone().unwrap_or_else(|| "".to_string())
                input_type="text"
                oninput=&self.oninput
                onblur=&self.onblur
                required=self.required
            />
        }
    }
}

impl PureComponent for PureUrlInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=&self.label
                value=&self.value.clone().unwrap_or_else(|| "".to_string())
                input_type="url"
                oninput=&self.oninput
                onblur=&self.onblur
                required=self.required
            />
        }
    }
}

impl PureComponent for PureDateInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=&self.label
                value=&self.value.clone().unwrap_or_else(|| "".to_string())
                input_type="date"
                oninput=&self.oninput
                onblur=&self.onblur
                required=self.required
            />
        }
    }
}

impl PureComponent for PureNumberInput {
    fn render(&self) -> VNode {
        html! {
            <FormInput
                label=&self.label
                value=&self.value.unwrap_or(0).to_string()
                input_type="number"
                oninput=&self.oninput
                onblur=&self.onblur
                required=self.required
            />
        }
    }
}

impl PureComponent for PureWorkTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        { for self.data.iter().map(|i| self.render_worktype(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureWorkStatusSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        { for self.data.iter().map(|i| self.render_workstatus(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureContributionTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|i| self.render_contributiontype(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PurePublicationTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|p| self.render_publicationtype(p)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureSubjectTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|p| self.render_subjecttype(p)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureSeriesTypeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|s| self.render_seriestype(s)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureLanguageCodeSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|l| self.render_languagecode(l)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureLanguageRelationSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                    >
                        { for self.data.iter().map(|l| self.render_languagerelation(l)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureBooleanSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                        onblur=&self.onblur
                    >
                        <option value=true selected=self.value>
                            { YES }
                        </option>
                        <option value=false selected=!self.value>
                            { NO }
                        </option>
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureImprintSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        <option value="">{"Select Imprint"}</option>
                        { for self.data.iter().map(|i| self.render_imprint(i)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PurePublisherSelect {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control is-expanded">
                    <div class="select is-fullwidth">
                    <select required=self.required onchange=&self.onchange>
                        <option value="">{"Select Publisher"}</option>
                        { for self.data.iter().map(|p| self.render_publisher(p)) }
                    </select>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureWorkTypeSelect {
    fn render_worktype(&self, w: &WorkTypeValues) -> VNode {
        if w.name == self.value {
            html! {
                <option value={&w.name} selected=true>
                    {&w.name}
                </option>
            }
        } else {
            html! {
                <option value={&w.name}>{&w.name}</option>
            }
        }
    }
}

impl PureWorkStatusSelect {
    fn render_workstatus(&self, w: &WorkStatusValues) -> VNode {
        if w.name == self.value {
            html! {
                <option value={&w.name} selected=true>
                    {&w.name}
                </option>
            }
        } else {
            html! {
                <option value={&w.name}>{&w.name}</option>
            }
        }
    }
}

impl PureContributionTypeSelect {
    fn render_contributiontype(&self, c: &ContributionTypeValues) -> VNode {
        if c.name == self.value {
            html! {
                <option value={&c.name} selected=true>
                    {&c.name}
                </option>
            }
        } else {
            html! {
                <option value={&c.name}>{&c.name}</option>
            }
        }
    }
}

impl PurePublicationTypeSelect {
    fn render_publicationtype(&self, p: &PublicationTypeValues) -> VNode {
        if p.name == self.value {
            html! {
                <option value={&p.name} selected=true>
                    {&p.name}
                </option>
            }
        } else {
            html! {
                <option value={&p.name}>{&p.name}</option>
            }
        }
    }
}

impl PureSubjectTypeSelect {
    fn render_subjecttype(&self, s: &SubjectTypeValues) -> VNode {
        if s.name == self.value {
            html! {
                <option value={&s.name} selected=true>
                    {&s.name}
                </option>
            }
        } else {
            html! {
                <option value={&s.name}>{&s.name}</option>
            }
        }
    }
}

impl PureSeriesTypeSelect {
    fn render_seriestype(&self, s: &SeriesTypeValues) -> VNode {
        if s.name == self.value {
            html! {
                <option value={&s.name} selected=true>
                    {&s.name}
                </option>
            }
        } else {
            html! {
                <option value={&s.name}>{&s.name}</option>
            }
        }
    }
}

impl PureLanguageCodeSelect {
    fn render_languagecode(&self, l: &LanguageCodeValues) -> VNode {
        if l.name == self.value {
            html! {
                <option value={&l.name} selected=true>
                    {&l.name}
                </option>
            }
        } else {
            html! {
                <option value={&l.name}>{&l.name}</option>
            }
        }
    }
}

impl PureLanguageRelationSelect {
    fn render_languagerelation(&self, l: &LanguageRelationValues) -> VNode {
        if l.name == self.value {
            html! {
                <option value={&l.name} selected=true>
                    {&l.name}
                </option>
            }
        } else {
            html! {
                <option value={&l.name}>{&l.name}</option>
            }
        }
    }
}

impl PureImprintSelect {
    fn render_imprint(&self, i: &Imprint) -> VNode {
        let value = &self.value.clone().unwrap_or_else(|| "".to_string());
        if &i.imprint_id == value {
            html! {
                <option value={&i.imprint_id} selected=true>
                    {&i.imprint_name}
                </option>
            }
        } else {
            html! {
                <option value={&i.imprint_id}>{&i.imprint_name}</option>
            }
        }
    }
}

impl PurePublisherSelect {
    fn render_publisher(&self, p: &Publisher) -> VNode {
        let value = &self.value.clone().unwrap_or_else(|| "".to_string());
        if &p.publisher_id == value {
            html! {
                <option value={&p.publisher_id} selected=true>
                    {&p.publisher_name}
                </option>
            }
        } else {
            html! {
                <option value={&p.publisher_id}>{&p.publisher_name}</option>
            }
        }
    }
}

impl PureComponent for PureLoader {
    fn render(&self) -> VNode {
        html! {
            <div class="hero is-medium">
                <div class="hero-body">
                    <div class="container has-text-centered">
                        <progress class="progress is-warning" max="100"></progress>
                    </div>
                </div>
            </div>
        }
    }
}

impl PureComponent for PureReloader {
    fn render(&self) -> VNode {
        html! {
            <div class="buttons has-addons is-centered">
                <button
                    class="button is-success is-large"
                    onclick=&self.onclick
                >
                    <span class="icon">
                        <i class="fas fa-sync"></i>
                    </span>
                    <span>{ RELOAD_BUTTON }</span>
                </button>
            </div>
        }
    }
}
