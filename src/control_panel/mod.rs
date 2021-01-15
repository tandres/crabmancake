use web_sys::Element;
use yew::prelude::*;
use rand::prelude::*;
use object_select::{ObjectSelect, ObjectOption};

mod object_select;

pub struct ControlPanelModel {
    link: ComponentLink<Self>,
    value: String,
    object_list: Vec<ObjectOption>,
    object_selected: String,
}

#[derive(Properties, Clone)]
pub struct ControlPanelProps {
    pub suffix: String,
}

pub enum Msg {
    AddObject(String),
    Hello,
    Select(String),
}

impl Component for ControlPanelModel {
    type Message = Msg;
    type Properties = ControlPanelProps;
    fn create(properties: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            value: format!("{}_{}", properties.suffix, "Hello"),
            object_list: Vec::new(),
            object_selected: "0".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddObject(value) => {
                log::info!("Adding object: {}", value);
                let display = format!("obj_{}", value);
                let object_option = ObjectOption {value, display};
                self.object_list.push(object_option);
                true
            },
            Msg::Hello => {
                log::info!("Hello {}", self.value);
                false
            },
            Msg::Select(s) => {
                log::info!("Selected fired: {}", s);
                self.object_selected = s;
                false
            },
        }
     }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <ObjectSelect onsignal=self.link.callback(|s| Msg::Select(s)) select_value={&self.object_selected} options={&self.object_list}/>
                <button onclick=self.link.callback(|_| Msg::AddObject(format!("{}", random::<u32>())))>{ "Add Object" }</button>
            </div>
        }
    }
}

impl ControlPanelModel {
    pub fn mount(element: &Element, props: ControlPanelProps) {
        App::<ControlPanelModel>::new().mount_with_props(element.clone(), props);
    }

    pub fn add_object(&self, object_id: u32) {
        self.link.send_message(Msg::AddObject(format!("{}", object_id)));
    }

    pub fn select_object(&self, object_value: String) {
        if self.object_list.iter().find(|v| v.value == object_value).is_some() {
            self.link.send_message(Msg::Select(object_value));
        }
    }
}
