#![feature(conservative_impl_trait, fs_read_write, inclusive_range_syntax, match_default_bindings)]

extern crate dotenv;
extern crate lalrpop_util;
extern crate serde_json as json;
extern crate serenity;
extern crate threadpool;
extern crate typemap;

#[macro_use]
extern crate serde_derive;

use serenity::prelude::*;
use serenity::model::prelude::*;

use std::env;
use std::sync::Arc;

pub mod state;
pub mod commands;
pub mod grammar;
pub mod framework;

struct BotGuildId;
impl typemap::Key for BotGuildId {
    type Value = GuildId;
}
struct BotUserId;
impl typemap::Key for BotUserId {
    type Value = UserId;
}
struct BotLogChannel;
impl typemap::Key for BotLogChannel {
    type Value = ChannelId;
}
impl typemap::Key for state::State {
    type Value = Arc<state::State>;
}

fn main() {
    let _ = dotenv::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("Please specify DISCORD_TOKEN");
    let guildid = env::var("BOT_GUILD_ID")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(GuildId)
        .expect("Please specify BOT_GUILD_ID");
    let logchan = env::var("BOT_LOG_CHANNEL")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(ChannelId)
        .expect("Please specify BOT_LOG_CHANNEL");

    let state = Arc::new(state::State::load());

    let mut client = Client::new(&token, Handler).unwrap();

    client.data.lock().insert::<BotGuildId>(guildid);
    client.data.lock().insert::<BotLogChannel>(logchan);
    client.data.lock().insert::<state::State>(state);

    client.with_framework(framework::BotFramework {});

    client.start().unwrap();
}

pub fn state(ctx: &Context) -> Arc<state::State> {
    ctx.data.lock().get::<state::State>().cloned().unwrap()
}

pub fn logchan(ctx: &Context) -> ChannelId {
    ctx.data.lock().get::<BotLogChannel>().cloned().unwrap()
}

pub fn bot_gid(ctx: &Context) -> GuildId {
    ctx.data.lock().get::<BotGuildId>().cloned().unwrap()
}

pub fn bot_uid(ctx: &Context) -> UserId {
    let uid = ctx.data.lock().get::<BotUserId>().cloned();
    uid.unwrap_or_else(|| {
        let curr_uid = serenity::http::get_current_user().unwrap().id;
        ctx.data.lock().insert::<BotUserId>(curr_uid);
        curr_uid
    })
}

pub fn log(ctx: &Context, msg: &str) {
    eprintln!("LOG: {}", msg);
    let logchan = logchan(ctx);
    match logchan.say(msg) {
        Ok(_) => {}
        Err(e) => eprintln!("Log failure: {}", e),
    }
}

struct Handler;
impl EventHandler for Handler {}
