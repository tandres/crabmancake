use thiserror::Error;
use wasm_bindgen::JsValue;

// pub struct ErrorDescription {
//     file: String,
//     line: String,
//     error: crate::error::CmcError,
//     msg: String,
// }

// macro_rules! report_error {
//     ( $sender:expr, $e:expr, $msg:expr  ) => {
//         let error_msg = crate::bus_manager::ErrorMsg {
//             file: file!(),
//             line: line!(),
//             error: $e,
//             msg: $msg
//         };

//         $sender.send(error_msg);
//     };
// }

// pub fn error_reporter(error_rx: Receiver<ErrorMsg>) -> Future<()> {
// }

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
    #[error("Element Error: {description}")]
    Element {
        element: web_sys::Element,
        description: String,
    },
    #[error("Gltf error: {error}")]
    Gltf {
        #[from]
        error: gltf::Error,
    },
    #[error("Image error: {error}")]
    Image {
        #[from]
        error: image::ImageError,
    },
    #[error("Reqwest error: {error}")]
    Reqwest {
        #[from]
        error: reqwest::Error,
    },
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

impl From<web_sys::Element> for CmcError {
    fn from(val: web_sys::Element) -> Self {
        CmcError::Element {
            element: val,
            description: String::from("Another thing you should have figured out"),
        }
    }
}
