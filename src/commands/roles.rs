use serenity::prelude::*;
use serenity::model::prelude::*;

use grammar::ast::Command;

pub fn scan_roles(ctx: &Context, msg: &Message, cmd: &Command) {
    let (high, low) = match cmd {
        Command::RescanRoles(high, low) => (high, low),
        _ => unreachable!(),
    };

    let guild = match msg.guild().map(|g| Guild::clone(&g.read())) {
        Some(g) => g,
        None => {
            ::log(ctx, "Oof, I couldn't find the guild???");
            let _ = msg.reply("Oops, something went wrong :( Ask a mod about it~");
            return;
        }
    };

    let state = ::state(ctx);
    let result = state.roles.rescan(&guild, high, low);
    match result {
        Ok(_) => {
            let _ = state.save();
            let taglist = state.roles.all_role_tags().collect::<Vec<_>>().join(", ");
            let _ = msg.reply(&format!("Done! New role list: {}", taglist));
        }
        Err(e) => {
            ::log(ctx, &e);
            let _ = msg.reply("oof. check the log, cutie~");
        }
    }
}

pub fn alias_role(ctx: &Context, msg: &Message, cmd: &Command) {
    let (alias, target) = match cmd {
        Command::AliasRole(alias, target) => (alias, target),
        _ => unreachable!(),
    };

    let (alias, target) = (role_tag(&alias), role_tag(&target));

    let state = ::state(ctx);
    state.roles.aliases.write().unwrap().insert(alias, target);
    let _ = state.save();

    let _ = msg.react("\u{1F44D}");
}

pub fn remove_alias(ctx: &Context, msg: &Message, cmd: &Command) {
    let alias = match cmd {
        Command::RemoveAlias(alias) => alias,
        _ => unreachable!(),
    };

    let alias = role_tag(&alias);

    let state = ::state(ctx);
    state.roles.aliases.write().unwrap().remove(&alias);
    let _ = state.save();

    let _ = msg.react("\u{1F44D}");
}

pub fn list_roles(ctx: &Context, msg: &Message, _cmd: &Command) {
    let state = ::state(ctx);

    let mut roles: Vec<_> = state
        .roles
        .roles
        .read()
        .unwrap()
        .iter()
        .map(|(n, _)| n.clone())
        .collect();

    roles.sort();

    let mut buf = String::from("All roles:");
    for role in roles {
        buf.push_str(&format!("\n{}", role));
    }
    let _ = msg.reply(&buf);
}

pub fn list_aliases(ctx: &Context, msg: &Message, _cmd: &Command) {
    let state = ::state(ctx);

    let mut aliases: Vec<_> = state
        .roles
        .aliases
        .read()
        .unwrap()
        .iter()
        .map(|(a, t)| (a.clone(), t.clone()))
        .collect();

    aliases.sort();

    let mut buf = String::from("All current aliases:");
    for (alias, target) in aliases {
        buf.push_str(&format!("\n{} => {}", alias, target));
    }
    let _ = msg.reply(&buf);
}

fn parse_roles(ctx: &Context, roles: &[String]) -> (Vec<RoleId>, Vec<String>) {
    let state = ::state(ctx);
    let all_roles = state.roles.all_roles();
    let all_aliases = state.roles.all_aliases();

    let mut ids = Vec::with_capacity(roles.len());
    let mut cant_find = vec![];
    for mut role in roles {
        if let Some(alias) = all_aliases.get(role) {
            role = alias;
        }

        if let Some(&id) = all_roles.get(role) {
            ids.push(id);
        } else {
            cant_find.push(role.clone());
        }
    }

    (ids, cant_find)
}

pub fn give_roles(ctx: &Context, msg: &Message, cmd: &Command) {
    let (target, roles) = match cmd {
        Command::GiveRoles { target, roles } => (*target, roles),
        _ => unreachable!(),
    };

    let mut target = match msg.guild_id().unwrap().member(target) {
        Ok(target) => target,
        Err(e) => {
            ::log(
                ctx,
                &format!(
                    "Wowzers! Error setting {}'s roles:\n{:?}",
                    msg.author.mention(),
                    e
                ),
            );
            let _ = msg.reply("Oops, something went wrong :( Ask a mod about it~");
            return;
        }
    };

    if roles.len() < 1 {
        let _ = msg.reply("You didn't list any roles...");
        return;
    }

    let (ids, cant_find) = parse_roles(ctx, roles);

    if cant_find.len() > 0 {
        let list = cant_find.join(", ");
        let _ = msg.reply(&format!(
            "I'm sorry, I couldn't find the following \
             roles. Make sure you spelled them right: {}",
            list
        ));
        return;
    }

    let _ = target.add_roles(&ids);
    let _ = msg.react("\u{1F44D}");
}

pub fn take_roles(ctx: &Context, msg: &Message, cmd: &Command) {
    let (target, roles) = match cmd {
        Command::TakeRoles { target, roles } => (*target, roles),
        _ => unreachable!(),
    };

    let mut target = match msg.guild_id().unwrap().member(target) {
        Ok(target) => target,
        Err(e) => {
            ::log(
                ctx,
                &format!(
                    "Wowzers! Error setting {}'s roles:\n{:?}",
                    msg.author.mention(),
                    e
                ),
            );
            let _ = msg.reply("Oops, something went wrong :( Ask a mod about it~");
            return;
        }
    };

    if roles.len() < 1 {
        let _ = msg.reply("You didn't list any roles...");
        return;
    }

    let (ids, cant_find) = parse_roles(ctx, roles);

    if cant_find.len() > 0 {
        let list = cant_find.join(", ");
        let _ = msg.reply(&format!(
            "I'm sorry, I couldn't find the following \
             roles. Make sure you spelled them right: {}",
            list
        ));
        return;
    }

    let _ = target.remove_roles(&ids);
    let _ = msg.react("\u{1F44D}");
}

fn role_tag(role: &str) -> String {
    role.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}
