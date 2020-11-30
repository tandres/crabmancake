use thiserror::Error;
use wasm_bindgen::JsValue;

pub type CmcResult<T> = Result<T, CmcError>;

#[derive(Error, Debug)]
pub enum CmcError {
    #[error("Missing value: {0}")]
    MissingVal(String),
    #[error("Conversion failed: {0}")]
    ConversionFail(String),
    #[error("Shader compilation failure: {log}")]
    ShaderCompile {
        log: String,
    },
    #[error("GL Program Link Failure: {log}")]
    ShaderLink {
        log: String,
    },
    #[error("JsValue Error: {description}")]
    JsValue {
        jsvalue: JsValue,
        description: String,
    },
    #[error("Gltf error: {error}")]
    Gltf {
        #[from]
        error: gltf::Error,
    },
    #[error("Png error: {error}")]
    Png {
        #[from]
        error: png::DecodingError,
    }
}

impl CmcError {
    pub fn missing_val<S: AsRef<str>>(msg: S) -> Self {
        Self::MissingVal(msg.as_ref().to_string())
    }

    pub fn conversion_failed<S: AsRef<str>>(msg: S) -> Self {
        Self::ConversionFail(msg.as_ref().to_string())
    }
}

impl From<CmcError> for JsValue {
    fn from(val: CmcError) -> Self {
        let msg = format!("{}", val);
        JsValue::from_str(&msg[..])
    }
}

impl From<JsValue> for CmcError {
    fn from(val: JsValue) -> Self {
        CmcError::JsValue {
            jsvalue: val,
            description: String::from("You should have figured this out"),
        }
    }
}
