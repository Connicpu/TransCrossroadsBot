use serenity::model::prelude::*;
use serenity::prelude::*;

use std::collections::HashSet;

use grammar::ast::Command;

pub fn scan_pronouns(ctx: &Context, msg: &Message, cmd: &Command) {
    let (high, low) = match cmd {
        Command::RescanPronouns(high, low) => (high, low),
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
    let result = state.pronouns.rescan(&guild, high, low);
    match result {
        Ok(_) => {
            let _ = state.save();
            let taglist = state
                .pronouns
                .all_role_tags()
                .collect::<Vec<_>>()
                .join(", ");
            let _ = msg.reply(&format!("Done! New pronoun list: {}", taglist));
        }
        Err(e) => {
            ::log(ctx, &e);
            let _ = msg.reply("oof. check the log, cutie~");
        }
    }
}

pub fn set_pronouns(ctx: &Context, msg: &Message, cmd: &mut Command) {
    let (target, pronouns) = match cmd {
        Command::SetPronouns { target, pronouns } => (*target, pronouns),
        _ => return,
    };

    let mut target = match msg.guild_id().unwrap().member(target) {
        Ok(target) => target,
        Err(e) => {
            ::log(
                ctx,
                &format!(
                    "Wowzers! Error setting {}'s pronouns:\n{:?}",
                    msg.author.mention(),
                    e
                ),
            );
            let _ = msg.reply("Oops, something went wrong :( Ask a mod about it~");
            return;
        }
    };

    if pronouns.len() < 1 {
        let _ = msg.reply(
            "You didn't list any pronouns. If you wanted the `No Pronouns` role, ask for `none`.",
        );
        return;
    }

    let state = ::state(ctx);
    let all_pronouns = state.pronouns.all_roles();

    let mut cant_find = Vec::new();
    for pronoun in pronouns.iter() {
        if all_pronouns.iter().find(|(p, _)| p == pronoun) == None {
            cant_find.push(pronoun.clone());
        }
    }

    if cant_find.len() > 0 {
        let list = cant_find.join(", ");
        let resp = format!(
            "Oops! Looks like I couldn't find some of your pronouns: {}\n\
            Make sure you typed them correctly, and if they don't exist ask an admin to create them!",
            list
        );
        let _ = msg.reply(&resp);
        return;
    }

    let primary_pronoun = pronouns.swap_remove(0);
    let mut secondary_pronouns = pronouns.iter().cloned().collect::<HashSet<_>>();

    let remove_roles = all_pronouns.iter().map(|&(_, id)| id).collect::<Vec<_>>();
    let mut add_roles = Vec::with_capacity(pronouns.len() + 1);

    let mut pronoun_iter = all_pronouns
        .into_iter()
        .skip_while(|(tag, _)| *tag != primary_pronoun);

    add_roles.push(pronoun_iter.next().unwrap().1);

    for (tag, id) in pronoun_iter {
        if secondary_pronouns.remove(&tag) {
            add_roles.push(id);
        }
    }

    if secondary_pronouns.len() > 0 {
        let list = secondary_pronouns
            .into_iter()
            .collect::<Vec<_>>()
            .join(", ");
        let resp = format!(
            "Oops! Looks like I can't currently give you {} while \
             keeping {} as your primary pronouns. Please ask an admin \
             to fix this if it's important to you! Otherwise, use a \
             different pronoun as your primary~",
            list, primary_pronoun,
        );
        let _ = msg.reply(&resp);
        return;
    }

    let res = target.remove_roles(&remove_roles);
    if let Err(err) = res {
        ::log(ctx, &format!("{:#?}", err));
        let _ = msg.reply("Oops! Something went wrong! Ask an admin about it!");
        return;
    }

    let res = target.add_roles(&add_roles);
    if let Err(err) = res {
        ::log(ctx, &format!("{:#?}", err));
        let _ = msg.reply("Oops! Something went wrong! Ask an admin about it!");
        return;
    }

    let _ = msg.react("\u{1F44D}");
}
