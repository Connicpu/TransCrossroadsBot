use std::sync::RwLock;

use rand::{thread_rng, Rng};
use serenity::model::id::{ChannelId, MessageId};

#[derive(Default, Serialize, Deserialize)]
pub struct Code {
    #[serde(skip)]
    code: RwLock<Option<String>>,
    #[serde(skip)]
    purge_data: RwLock<Option<(ChannelId, Vec<MessageId>)>>,
}

impl Code {
    pub fn issue(&self) -> String {
        let code: String = Some('t')
            .into_iter()
            .chain(
                thread_rng()
                    .gen_ascii_chars()
                    .take(24)
                    .flat_map(|c| c.to_lowercase()),
            )
            .collect();
        *self.code.write().unwrap() = Some(code.clone());
        code
    }

    pub fn take(&self) -> Option<String> {
        self.code.write().unwrap().take()
    }

    pub fn set_purge(&self, purge: Option<(ChannelId, Vec<MessageId>)>) {
        *self.purge_data.write().unwrap() = purge
    }

    pub fn take_purge(&self) -> Option<(ChannelId, Vec<MessageId>)> {
        self.purge_data.write().unwrap().take()
    }
}
