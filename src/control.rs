use crate::{CmcClient, error::{CmcError, CmcResult}, network::Sender};
use web_sys::{Document, Element, HtmlButtonElement, HtmlElement, HtmlLabelElement, Event, EventTarget, HtmlInputElement, HtmlOptionElement, HtmlSelectElement};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use js_sys::Function;
use std::collections::BTreeMap;
use std::rc::Rc;

struct ControlOption {
    id: u32,
    text: String,
    value: String,
    html: HtmlOptionElement,
}

pub struct ControlSelect {
    document: Rc<Document>,
    parent: Rc<Element>,
    div: HtmlElement,
    select: HtmlSelectElement,
    label: Option<HtmlLabelElement>,
    options: BTreeMap<String, ControlOption>,
    callback: Closure<dyn FnMut(Event)>,
}

impl ControlSelect {
    pub fn new(document: &Rc<Document>, parent: &Rc<Element>, label: Option<&str>, name: &str) -> CmcResult<Self> {
        let callback: Box<dyn FnMut(Event)> = Box::new(move |event: Event| {
            if let Some(target) = event.target() {
                if let Some(target_inner) = target.dyn_ref::<HtmlSelectElement>() {
                    log::info!("Select event: {:?}", target_inner.value());
                }
            }
        });
        let callback = Closure::wrap(callback);
        let options = BTreeMap::new();
        let base = document.create_element("select")?;
        base.set_attribute("name", name)?;
        base.set_attribute("id", name)?;
        let select: HtmlSelectElement = base.dyn_into::<HtmlSelectElement>()?;
        select.add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())?;

        let div: HtmlElement = document.create_element("div")?.dyn_into::<HtmlElement>()?;
        let label = if let Some(label) = label {
            let label_html = document.create_element("label")?.dyn_into::<HtmlLabelElement>()?;
            label_html.set_inner_html(label);
            div.append_child(&label_html)?;
            Some(label_html)
        } else {
            None
        };
        div.append_child(&select)?;
        Ok(Self {
            document: document.clone(),
            parent: parent.clone(),
            div,
            select,
            label,
            options,
            callback,
        })
    }

    pub fn append_to_parent(&self) -> CmcResult<()> {
        self.parent.append_child(&self.div)?;
        Ok(())
    }

    pub fn remove_option(&mut self, value: &str) -> CmcResult<()> {
        let opt = self.options.remove(value).ok_or_else(|| CmcError::missing_val("Could not find optioncontrol element"))?;
        self.select.remove_child(&opt.html)?;
        Ok(())
    }

    pub fn add_option(&mut self, id: u32, name: &str, value: &str) -> CmcResult<()> {
        let html = HtmlOptionElement::new_with_text_and_value(name, value)?;
        let opt = ControlOption {
            id,
            text: String::from(name),
            value: String::from(value),
            html
        };
        self.select.append_child(&opt.html)?;
        self.options.insert(String::from(value), opt);
        Ok(())
    }

}

pub struct ControlButton {
    document: Rc<Document>,
    parent: Rc<Element>,
    div: HtmlElement,
    button: HtmlButtonElement,
    label: Option<HtmlLabelElement>,
    button_text: String,
    callback: Closure<dyn FnMut(Event)>,
}

impl ControlButton {
    pub fn new(document: &Rc<Document>, parent: &Rc<Element>, label: Option<&str>, button_text: &str, sender: Sender<(usize, bool)>) -> CmcResult<Self> {
        let callback: Box<dyn FnMut(Event)> = Box::new(move |event: Event| {
            sender.send((1, true));
            log::info!("button pressed!");
        });
        let callback = Closure::wrap(callback);
        let button_text = button_text.to_string();

        let base = document.create_element("button")?;
        base.set_inner_html(&button_text);
        let button: HtmlButtonElement = base.dyn_into::<HtmlButtonElement>()?;
        button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;

        let div: HtmlElement = document.create_element("div")?.dyn_into::<HtmlElement>()?;
        let label = if let Some(label) = label {
            let label_html = document.create_element("label")?.dyn_into::<HtmlLabelElement>()?;
            label_html.set_inner_html(label);
            div.append_child(&label_html)?;
            Some(label_html)
        } else {
            None
        };
        div.append_child(&button)?;
        Ok(ControlButton {
            document: document.clone(),
            parent: parent.clone(),
            div,
            button,
            label,
            button_text,
            callback,
        })
    }

    pub fn append_to_parent(&self) -> CmcResult<()> {
        self.parent.append_child(&self.div)?;
        Ok(())
    }
}

// fn create_slider<F>(document: &Document, element: &Element, label: &str, range: std::ops::Range<f32>, start: f32, mut func: F) -> Result<(), JsValue>
// where
//     F: FnMut(f64) + 'static,
// {

//     let html_label = document.create_element("p")?;
//     html_label.set_inner_html(label);
//     let base = document.create_element("input")?;
//     base.set_attribute("type", "range")?;
//     base.set_attribute("min", &range.start.to_string())?;
//     base.set_attribute("max", &range.end.to_string())?;
//     base.set_attribute("value", &start.to_string())?;
//     base.set_attribute("label", label)?;
//     base.set_attribute("class", "inputSlider")?;
//     let html_input: HtmlInputElement = base.dyn_into::<HtmlInputElement>()?;
//     let handler = move |event: web_sys::Event| {
//         if let Some(target) = event.target() {
//             if let Some(target_inner) = target.dyn_ref::<HtmlInputElement>() {
//                 let value = target_inner.value_as_number();
//                 func(value);
//             }
//         }
//     };
//     let handler = Closure::wrap(Box::new(handler) as Box<dyn FnMut(_)>);
//     html_input.add_event_listener_with_callback("input", &Function::from(handler.into_js_value()))?;
//     element.append_child(&html_label)?;
//     element.append_child(&html_input)?;
//     Ok(())
// }

