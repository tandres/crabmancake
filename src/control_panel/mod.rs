use web_sys::Element;
use rand::prelude::*;
use yew::prelude::*;
use object_select::ObjectSelect;

mod object_select;

pub struct ControlPanelModel {
    link: ComponentLink<Self>,
    value: String,
    object_list: Vec<(String, String)>,
}

#[derive(Properties, Clone)]
pub struct ControlPanelProps {
    pub suffix: String,
}

pub enum Msg {
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
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Hello => {
                log::info!("{}", self.value);
                let new_object = format!("obj_{}", random::<i32>());
                self.object_list.push((new_object.clone(), new_object));
            },
            Msg::Select(s) => {
                log::info!("Selected fired: {}", s);
            },
        }
        true
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
                <ObjectSelect onsignal=self.link.callback(|s| Msg::Select(s)) options={&self.object_list}/>
                <button onclick=self.link.callback(|_| Msg::Hello)>{ "Add Object" }</button>
            </div>
        }
    }
}

impl ControlPanelModel {
    pub fn mount(element: &Element, props: ControlPanelProps) {
        App::<ControlPanelModel>::new().mount_with_props(element.clone(), props);
    }
}
