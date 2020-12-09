#[derive(Clone, Debug)]
pub struct KeyState {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }

    pub fn set_key(&mut self, key: String) {
        match key.as_ref() {
            "KeyW" => self.forward = true,
            "KeyS" => self.backward = true,
            "KeyA" => self.left = true,
            "KeyD" => self.right = true,
            k => log::warn!("Unhandled key: {}", k),
        }
    }

    pub fn clear(&mut self) {
        self.forward = false;
        self.backward = false;
        self.left = false;
        self.right = false;
    }
}


