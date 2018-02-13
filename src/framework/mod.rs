use serenity::framework::Framework;
use serenity::prelude::*;
use serenity::model::prelude::*;
use threadpool::ThreadPool;

use std::mem::replace;

use {bot_gid, bot_uid, grammar};

pub struct BotFramework {}

impl Framework for BotFramework {
    fn dispatch(&mut self, ctx: Context, mut msg: Message, pool: &ThreadPool) {
        let bot_uid = bot_uid(&ctx);
        if msg.author.id == bot_uid {
            return;
        }

        let msg_gid = msg.guild_id();

        if msg_gid.is_none() {
            let _ = msg.reply(
                "Hey, it's cool that you're trying to PM me, but that's not gonna work sorry~",
            );
            return;
        }

        if msg_gid != Some(bot_gid(&ctx)) {
            return;
        }

        if msg.mentions.iter().find(|m| m.id == bot_uid) == None {
            return;
        }

        pool.execute(move || {
            let clen = msg.content.len();
            let content = replace(&mut msg.content, String::with_capacity(clen));
            let first_mention = content.find("<@").unwrap();
            msg.content.push_str(&content[first_mention..]);
            println!("{}", msg.content);

            let cmdmember = match msg.member() {
                Some(m) => m,
                None => return,
            };

            let cmduser = msg.author.id;
            let (bot_mntn, mut cmd) = match grammar::parse_command(cmduser, &msg.content) {
                Ok(cmd) => cmd,
                Err(_) => {
                    let _ = msg.react("\u{1F615}");
                    return;
                }
            };

            if bot_mntn != bot_uid {
                return;
            }

            if !cmd.is_authorized(cmduser, &cmdmember) {
                let _ = msg.react("\u{274C}");
                return;
            }

            cmd.execute(&ctx, &msg);
        });
    }
}
