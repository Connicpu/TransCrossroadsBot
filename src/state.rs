use serenity::model::prelude::*;
use json;

use std::{fs, io};
use std::sync::RwLock;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    pub pronouns: PronounRoles,
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

#[derive(Default, Serialize, Deserialize)]
pub struct PronounRoles {
    pub roles: RwLock<Vec<(String, RoleId)>>,
}

impl PronounRoles {
    pub fn all_roles(&self) -> Vec<(String, RoleId)> {
        self.roles.read().unwrap().clone()
    }

    pub fn all_role_tags(&self) -> impl Iterator<Item = String> {
        self.all_roles().into_iter().map(|(v, _)| v)
    }

    pub fn all_role_ids(&self) -> impl Iterator<Item = RoleId> {
        self.all_roles().into_iter().map(|(_, v)| v)
    }

    pub fn rescan(&self, guild: &Guild, high: &str, low: &str) -> Result<(), String> {
        let mut roles = guild
            .roles
            .values()
            .filter_map(|r| {
                let tag = pronoun_tag(&r);
                tag.map(|tag| (r.position, r.id, tag))
            })
            .collect::<Vec<_>>();

        roles.sort_by_key(|&(p, _, _)| -p);

        let first = roles
            .iter()
            .enumerate()
            .find(|&(_, &(_, _, ref tag))| tag == high)
            .map(|(i, _)| i)
            .ok_or_else(|| format!("Pronouns `{}` not found on the list", high))?;
        let last = roles
            .iter()
            .enumerate()
            .rev()
            .find(|&(_, &(_, _, ref tag))| tag == low)
            .map(|(i, _)| i)
            .ok_or_else(|| format!("Pronouns `{}` not found on the list", low))?;

        if first > last {
            return Err(
                "Those pronouns are listed in the wrong order. Are you sure you did that right?"
                    .to_string(),
            );
        }

        let mut p_roles = self.roles.write().unwrap();
        p_roles.clear();
        p_roles.reserve(last - first + 1);
        p_roles.extend(
            roles
                .into_iter()
                .skip(first)
                .take(last - first + 1)
                .map(|(_, id, tag)| (tag, id)),
        );

        Ok(())
    }
}

fn pronoun_tag(role: &Role) -> Option<String> {
    if role.name == "Any Pronouns" {
        return Some("any".into());
    }
    if role.name == "No Pronouns" {
        return Some("none".into());
    }

    let mut parts = role.name.split('/');

    let first_part = parts
        .next()?
        .chars()
        .flat_map(|c| c.to_lowercase())
        .collect::<String>();
    let second_part = parts
        .next()?
        .chars()
        .flat_map(|c| c.to_lowercase())
        .collect::<String>();

    Some(format!("{}/{}", first_part, second_part))
}
