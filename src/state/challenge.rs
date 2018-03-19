use std::sync::RwLock;
use rand::{Rng, thread_rng};

#[derive(Default, Serialize, Deserialize)]
pub struct Code {
    code: RwLock<Option<String>>,
}

impl Code {
    pub fn issue(&self) -> String {
        let code: String = thread_rng().gen_ascii_chars().take(24).collect();
        *self.code.write().unwrap() = Some(code.clone());
        code
    }

    pub fn get(&self) -> Option<String> {
        self.code.read().unwrap().clone()
    }
}
