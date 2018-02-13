use serenity::model::prelude::*;

use std::sync::RwLock;

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
