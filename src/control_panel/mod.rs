use web_sys::Element;
use yew::prelude::*;
use rand::prelude::*;
use object_select::{ObjectSelect, ObjectOption};
use crate::bus::Sender;
use crate::bus_manager::{BusManager, UiMsg};
use std::rc::Rc;
use crate::uid::{self, Uid};

mod object_select;

pub struct ControlPanelModel {
    link: ComponentLink<Self>,
    uimsg_sender: Sender<UiMsg>,
    object_list: Vec<ObjectOption>,
    object_selected: String,
}

#[derive(Properties, Clone)]
pub struct ControlPanelProps {
    pub bus_manager: Rc<BusManager>,
}

pub enum Msg {
    AddObject(uid::Uid),
    Select(String),
}

impl Component for ControlPanelModel {
    type Message = Msg;
    type Properties = ControlPanelProps;
    fn create(properties: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            uimsg_sender: properties.bus_manager.ui.new_sender(),
            object_list: Vec::new(),
            object_selected: String::from(Uid::invalid()),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddObject(value) => {
                log::info!("Adding object: {}", value);
                let display = format!("{}", value);
                let object_option = ObjectOption {value: value.clone(), display};
                self.object_list.push(object_option);
                self.uimsg_sender.send(UiMsg::NewObject(value));
                true
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
                <button onclick=self.link.callback(|_| Msg::AddObject(uid::get_new_uid()))>{ "Add Object" }</button>
            </div>
        }
    }
}

impl ControlPanelModel {
    pub fn mount(element: &Element, props: ControlPanelProps) -> ComponentLink<Self> {
        App::<ControlPanelModel>::new().mount_with_props(element.clone(), props)
    }

    // pub fn add_object(&self, object_id: u32) {
    //     self.link.send_message(Msg::AddObject(format!("{}", object_id)));
    // }

    // pub fn select_object(&self, object_value: String) {
    //     if self.object_list.iter().find(|v| v.value == object_value).is_some() {
    //         self.link.send_message(Msg::Select(object_value));
    //     }
    // }
}