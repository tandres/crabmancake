use yew::prelude::*;
use crate::uid::Uid;
use std::num::ParseFloatError;

pub struct ObjectAdd {
    link: ComponentLink<Self>,
    onsignal: Callback<(Uid, [f32; 3])>,
    position_raw: [String; 3],
}

#[derive(Properties, Clone)]
pub struct ObjectAddProps {
    pub onsignal: Callback<(Uid, [f32; 3])>,
}

pub enum Msg {
    Update(usize, InputData),
    Submit,
}

impl Component for ObjectAdd {
    type Message = Msg;
    type Properties = ObjectAddProps;

    fn create(properties: Self::Properties, link: ComponentLink<Self>) -> Self {
        let position_raw = ["0".to_string(), "0".to_string(), "0".to_string()];
        Self {
            link,
            onsignal: properties.onsignal,
            position_raw,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update(index, val) => {
                log::info!("Val: {:?}", val);
                self.position_raw[index] = val.value;
                true
            },
            Msg::Submit => {
                log::info!("Submit");
                let parsed : Result<Vec<f32>, ParseFloatError> = self.position_raw.iter().map(|s| s.parse::<f32>()).collect();
                if let Ok(parsed) = parsed {
                    let position = [parsed[0], parsed[1], parsed[2]];
                    self.onsignal.emit((Uid::new(), position));
                }
                true
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <label for="x">{"x"}</label>
                <input type="text" name="x" oninput=self.link.callback(|v| Msg::Update(0, v)) value={&self.position_raw[0]}/>
                <label for="y">{"y"}</label>
                <input type="text" name="y" oninput=self.link.callback(|v| Msg::Update(1, v)) value={&self.position_raw[1]}/>
                <label for="z">{"z"}</label>
                <input type="text" name="z" oninput=self.link.callback(|v| Msg::Update(2, v)) value={&self.position_raw[2]}/>
                <button onclick=self.link.callback(|_| Msg::Submit)>{"Add Object"}</button>
            </div>
        }
    }

}
