use serenity::model::prelude::*;

use std::sync::RwLock;
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize)]
pub struct Roles {
    pub roles: RwLock<HashMap<String, RoleId>>,
    pub aliases: RwLock<HashMap<String, String>>,
}

impl Roles {
    pub fn all_roles(&self) -> HashMap<String, RoleId> {
        HashMap::clone(&self.roles.read().unwrap())
    }

    pub fn all_role_tags(&self) -> impl Iterator<Item = String> {
        self.all_roles().into_iter().map(|(v, _)| v)
    }

    pub fn all_role_ids(&self) -> impl Iterator<Item = RoleId> {
        self.all_roles().into_iter().map(|(_, v)| v)
    }

    pub fn all_aliases(&self) -> HashMap<String, String> {
        HashMap::clone(&self.aliases.read().unwrap())
    }

    pub fn rescan(&self, guild: &Guild, high: &str, low: &str) -> Result<(), String> {
        let mut roles = guild
            .roles
            .values()
            .map(|r| {
                let tag = role_tag(&r);
                (r.position, r.id, tag)
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

fn role_tag(role: &Role) -> String {
    role.name
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}
