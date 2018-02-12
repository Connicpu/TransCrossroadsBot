use serenity::framework::Framework;
use serenity::prelude::*;
use serenity::model::prelude::*;
use threadpool::ThreadPool;

use {bot_gid, bot_uid, grammar};

pub struct BotFramework {}

impl Framework for BotFramework {
    fn dispatch(&mut self, ctx: Context, msg: Message, pool: &ThreadPool) {
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
            let cmdmember = match msg.member() {
                Some(m) => m,
                None => return,
            };

            let cmduser = msg.author.id;
            let (bot_mntn, mut cmd) = match grammar::parse_command(cmduser, &msg.content) {
                Ok(cmd) => cmd,
                Err(_) => {
                    let _ = msg.reply("?");
                    return;
                }
            };

            if bot_mntn != bot_uid {
                return;
            }

            if !cmd.is_authorized(cmduser, &cmdmember) {
                let _ = msg.reply(&format!(
                    "I'm sorry {}, but I'm afraid I can't let you do that...",
                    msg.author.name
                ));
                return;
            }

            cmd.execute(&ctx, &msg);
        });
    }
}
