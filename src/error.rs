use thiserror::Error;
use wasm_bindgen::JsValue;


pub type CmcResult<T> = Result<T, CmcError>;

#[derive(Error, Debug)]
pub enum CmcError {
    #[error("Missing value: {0}")]
    MissingVal(String),
}

impl CmcError {
    pub fn missing_val<S: AsRef<str>>(msg: S) -> Self {
        Self::MissingVal(msg.as_ref().to_string())
    }
}

impl From<CmcError> for JsValue {
    fn from(val: CmcError) -> Self {
        let msg = format!("{}", val);
        JsValue::from_str(&msg[..])
    }
}
