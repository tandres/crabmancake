use std::{fmt, sync::Mutex};
use lazy_static::lazy_static;

lazy_static! {
   static ref LAST_UID: Mutex<u32> = Mutex::new(1);
}

#[derive(Clone, PartialEq, Debug)]
pub struct Uid {
    inner: u32
}

impl Uid {
    pub fn invalid() -> Uid {
        Uid { inner: 0 }
    }
}

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "uid_{}", self.inner)
    }
}

impl From<Uid> for String {
    fn from(item: Uid) -> String {
        format!("{}", item.inner)
    }
}

impl From<&Uid> for String {
    fn from(item: &Uid) -> String {
        format!("{}", item.inner)
    }
}

pub fn get_new_uid() -> Uid {
    let mut last = LAST_UID.lock().unwrap();
    let res = Uid { inner: *last };
    *last += 1;
    res
}



