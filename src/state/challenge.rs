use std::sync::RwLock;

#[derive(Default, Serialize, Deserialize)]
pub struct Code {
    pub code: RwLock<Option<String>>,
}
