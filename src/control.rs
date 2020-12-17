use crate::{CmcClient, error::{CmcError, CmcResult}};
use web_sys::{Document, Element, HtmlElement, HtmlLabelElement, Event, EventTarget, HtmlInputElement, HtmlOptionElement, HtmlSelectElement};
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
