use std::collections::HashMap;
use std::sync::Mutex;

use serenity::model::id::UserId;

#[derive(Serialize, Deserialize, Default)]
pub struct Counts {
    counts: Mutex<HashMap<UserId, u32>>,
}

impl Counts {
    pub fn increment(&self, user: UserId) -> u32 {
        let mut map = self.counts.lock().unwrap();
        let count = map.entry(user).or_insert(0);
        *count += 1;
        *count
    }

    pub fn get(&self, user: UserId) -> u32 {
        self.counts.lock().unwrap().get(&user).cloned().unwrap_or(0)
    }
}
