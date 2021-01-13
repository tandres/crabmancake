use yew::prelude::*;
use yew::events::ChangeData;

pub struct ObjectSelect {
    link: ComponentLink<Self>,
    label: String,
    options: Vec<(String, String)>,
    onsignal: Callback<String>,
}

pub enum Msg {
    OnChange(ChangeData),
}

#[derive(Clone, PartialEq, Properties)]
pub struct ObjectSelectProps {
    #[prop_or_default]
    pub label: String,
    pub onsignal: Callback<String>,
    pub options: Vec<(String, String)>,
}

impl Component for ObjectSelect {
    type Message = Msg;
    type Properties = ObjectSelectProps;
    fn create(properties: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            label: properties.label,
            options: vec![("0".to_string(), "None".to_string())],
            onsignal: properties.onsignal,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::OnChange(change_data) => {
                match change_data {
                    ChangeData::Value(data) => {
                        self.onsignal.emit(data);
                    },
                    ChangeData::Select(select_element) => {
                        self.onsignal.emit(select_element.value());
                    },
                    ChangeData::Files(_) => {
                        log::warn!("Got file list event unexpectedly");
                    },
                }
            }
        }
        true
     }

    fn change(&mut self, mut props: Self::Properties) -> ShouldRender {
        if self.options.len() != props.options.len() + 1 {
            self.options = vec![("0".to_string(), "None".to_string())];
            self.options.append(&mut props.options);
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <label> {&self.label}
                <select onchange=self.link.callback(|s| Msg::OnChange(s))>
                    {self.options.iter().map(Self::render_option).collect::<Html>()}
                </select>
            </label>
        }
    }
}

impl ObjectSelect {
    fn render_option(opt: &(String, String)) -> Html {
        html! {
            <option value={&opt.0}>{&opt.1}</option>
        }
    }
}
