use json;

use std::{fs, io};

pub mod pronouns;
pub mod roles;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub pronouns: pronouns::PronounRoles,
    #[serde(default)]
    pub roles: roles::Roles,
}

const STATE_FILE: &str = "state.json";
const BACKUP_STATE: &str = "old_state.json";

impl State {
    pub fn load() -> State {
        State::try_load().unwrap_or_else(|e| {
            println!("Couldn't load state: {:?}", e);

            // Make a backup of the state
            let _ = fs::copy(STATE_FILE, BACKUP_STATE);

            // Create and save a new state
            let state = State::default();
            let _ = state.save();
            state
        })
    }

    fn try_load() -> io::Result<State> {
        let data = fs::read(STATE_FILE)?;
        Ok(json::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    }

    pub fn save(&self) -> io::Result<()> {
        let data = json::to_vec(self).unwrap();
        fs::write(STATE_FILE, data)?;
        Ok(())
    }
}


