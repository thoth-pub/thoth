use yew::virtual_dom::VNode;
use yew::html;
use yew::Properties;
use yewtil::Pure;
use yewtil::PureComponent;

pub type FormInput = Pure<PureInput>;
pub type FormTextarea = Pure<PureTextarea>;
pub type FormTextInput = Pure<PureTextInput>;
pub type FormUrlInput = Pure<PureUrlInput>;
pub type FormDateInput = Pure<PureDateInput>;
pub type FormNumberInput = Pure<PureNumberInput>;
pub type Loader = Pure<PureLoader>;

#[derive(Clone, PartialEq, Properties)]
pub struct PureInput {
    pub label: String,
    pub value: String,
    pub input_type: String,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureTextarea {
    pub label: String,
    pub value: Option<String>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureTextInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureUrlInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureDateInput {
    pub label: String,
    pub value: Option<String>,
    #[prop_or(false)]
    pub required: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PureNumberInput {
    pub label: String,
    pub value: Option<i32>,
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
                        required={ self.required }
                    >
                        {&self.value.clone().unwrap_or("".to_string())}
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
                value=&self.value.clone().unwrap_or("".to_string())
                input_type="text"
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
                value=&self.value.clone().unwrap_or("".to_string())
                input_type="url"
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
                value=&self.value.clone().unwrap_or("".to_string())
                input_type="date"
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
                required=self.required
            />
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
