use thoth_api::models::contributor::ContributionType;
use thoth_api::models::work::WorkStatus;
use thoth_api::models::work::WorkType;
use yew::html;
use yew::virtual_dom::VNode;
use yew::Callback;
use yew::ChangeData;
use yew::FocusEvent;
use yew::InputData;
use yew::Properties;
use yewtil::Pure;
use yewtil::PureComponent;

use crate::api::models::ContributionTypeValues;
use crate::api::models::Imprint;
use crate::api::models::WorkStatusValues;
use crate::api::models::WorkTypeValues;

pub type FormInput = Pure<PureInput>;
pub type FormTextarea = Pure<PureTextarea>;
pub type FormTextInput = Pure<PureTextInput>;
pub type FormUrlInput = Pure<PureUrlInput>;
pub type FormDateInput = Pure<PureDateInput>;
pub type FormNumberInput = Pure<PureNumberInput>;
pub type FormWorkTypeSelect = Pure<PureWorkTypeSelect>;
pub type FormWorkStatusSelect = Pure<PureWorkStatusSelect>;
pub type FormContributionTypeSelect = Pure<PureContributionTypeSelect>;
pub type FormImprintSelect = Pure<PureImprintSelect>;
pub type Loader = Pure<PureLoader>;

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
pub struct PureLoader {}

impl PureComponent for PureInput {
    fn render(&self) -> VNode {
        html! {
            <div class="field">
                <label class="label">{ &self.label }</label>
                <div class="control">
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
                <div class="control">
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
                <div class="control">
                    <div class="select">
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
                <div class="control">
                    <div class="select">
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
                <div class="control">
                    <div class="select">
                    <select
                        required=self.required
                        onchange=&self.onchange
                        onblur=&self.onblur
                    >
                        { for self.data.iter().map(|i| self.render_contributiontype(i)) }
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
                <div class="control">
                    <div class="select">
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
