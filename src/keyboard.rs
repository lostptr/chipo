pub struct Keyboard {
    keys: [u8; 16]
}

impl Keyboard {

    pub fn new() -> Self {
        Keyboard {
            keys: [0; 16]
        }
    }

    pub fn is_key_pressed(&self, key_code: u8) -> bool {
        false
    }
}