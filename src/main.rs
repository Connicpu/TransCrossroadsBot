#![feature(conservative_impl_trait, fs_read_write, inclusive_range_syntax, match_default_bindings)]

extern crate dotenv;
extern crate lalrpop_util;
extern crate rand;
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
pub struct StaffAlertData {
    admin_channel: ChannelId,
    mod_channel: ChannelId,
    front_door: ChannelId,
    mod_call: RoleId,
}
impl typemap::Key for StaffAlertData {
    type Value = Arc<StaffAlertData>;
}

fn env_token<F, R>(token: &str, f: F) -> R
where
    F: Fn(u64) -> R,
{
    env::var(token)
        .ok()
        .and_then(|s| s.parse().ok())
        .map(f)
        .expect(token)
}

fn main() {
    let _ = dotenv::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("Please specify DISCORD_TOKEN");
    let guildid = env_token("BOT_GUILD_ID", GuildId);
    let logchan = env_token("BOT_LOG_CHANNEL", ChannelId);
    let admin_channel = env_token("ADMIN_CHANNEL", ChannelId);
    let mod_channel = env_token("MOD_CHANNEL", ChannelId);
    let front_door = env_token("FRONT_DOOR", ChannelId);
    let mod_call = env_token("MOD_CALL", RoleId);

    let staff_alert = Arc::new(StaffAlertData {
        admin_channel,
        mod_channel,
        front_door,
        mod_call,
    });

    let state = Arc::new(state::State::load());

    let mut client = Client::new(&token, Handler).unwrap();

    client.data.lock().insert::<BotGuildId>(guildid);
    client.data.lock().insert::<BotLogChannel>(logchan);
    client.data.lock().insert::<state::State>(state);
    client.data.lock().insert::<StaffAlertData>(staff_alert);

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

pub fn logres<T, E>(ctx: &Context, result: Result<T, E>)
where
    E: std::fmt::Debug,
{
    match result {
        Err(e) => log(ctx, &format!("{:#?}", e)),
        Ok(_) => (),
    }
}

pub fn staff_alert(ctx: &Context) -> Arc<StaffAlertData> {
    ctx.data.lock().get::<StaffAlertData>().cloned().unwrap()
}

struct Handler;
impl EventHandler for Handler {
    fn guild_member_addition(&self, context: Context, guild: GuildId, _: Member) {
        if guild != bot_gid(&context) {
            return;
        }

        let staff_alert = staff_alert(&context);
        let _ = staff_alert.mod_channel.say(format!(
            "Hey {modcall}, there's a new user in {frontdoor}~",
            modcall = staff_alert.mod_call.mention(),
            frontdoor = staff_alert.front_door.mention(),
        ));
    }

    fn guild_member_removal(
        &self,
        context: Context,
        guild: GuildId,
        user: User,
        _: Option<Member>,
    ) {
        if guild != bot_gid(&context) {
            return;
        }

        log(
            &context,
            &format!("{}#{} left the server", user.name, user.discriminator),
        );
    }
}
