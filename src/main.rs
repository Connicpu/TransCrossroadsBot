#![feature(conservative_impl_trait, fs_read_write, inclusive_range_syntax, match_default_bindings)]

#[macro_use]
extern crate lazy_static;

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
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::fmt::Write;

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
    the_void: ChannelId,
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
    let the_void = env_token("THE_VOID", ChannelId);

    let _ = DELETE_QUEUE.lock().unwrap().send(Err(the_void));

    let staff_alert = Arc::new(StaffAlertData {
        admin_channel,
        mod_channel,
        front_door,
        mod_call,
        the_void,
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

lazy_static! {
    static ref DELETE_QUEUE: Arc<Mutex<Sender<Result<MessageId, ChannelId>>>> = {
        let (tx, rx) = channel::<Result<MessageId, ChannelId>>();
        std::thread::spawn(move || {
            let channel = rx.recv().unwrap().unwrap_err();
            let mut buf = vec![];
            while let Ok(Ok(first)) = rx.recv() {
                buf.push(first);
                std::thread::sleep(std::time::Duration::from_secs(1));
                while let Ok(Ok(next)) = rx.try_recv() {
                    buf.push(next);
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
                let _ = channel.delete_messages(&buf);
                buf.clear();
            }
        });

        Arc::new(Mutex::new(tx))
    };
}

struct Handler;
impl EventHandler for Handler {
    fn message(&self, context: Context, msg: Message) {
        let staff_alert = staff_alert(&context);
        if msg.channel_id != staff_alert.the_void {
            return;
        }

        let channel = match msg.channel_id.get() {
            Ok(Channel::Guild(gc)) => gc,
            _ => return,
        };

        let guild = match serenity::CACHE
            .read()
            .guilds
            .get(&channel.read().guild_id)
            .cloned()
        {
            Some(guild) => guild,
            None => return,
        };

        let permissions = guild.read().member_permissions(msg.author.id);
        let is_admin = permissions & Permissions::ADMINISTRATOR == Permissions::ADMINISTRATOR;

        if msg.content.starts_with("ADMIN:") && is_admin {
            return;
        }

        let _ = DELETE_QUEUE.lock().unwrap().send(Ok(msg.id));
    }

    fn guild_member_addition(&self, context: Context, guild: GuildId, member: Member) {
        if guild != bot_gid(&context) {
            return;
        }

        let staff_alert = staff_alert(&context);
        let state = state(&context);

        let mut message = format!(
            "Hey {modcall}, there's a new user in {frontdoor}~",
            modcall = staff_alert.mod_call.mention(),
            frontdoor = staff_alert.front_door.mention(),
        );

        let times = state.leave_counts.get(member.user.read().id);
        if times > 0 {
            let _ = write!(
                &mut message,
                " (they're back for the {}{} time)",
                times,
                cardinality(times)
            );
        }

        let _ = staff_alert.mod_channel.say(message);
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

        let state = state(&context);
        let times = state.leave_counts.increment(user.id);
        let _ = state.save();

        log(
            &context,
            &format!(
                "{}#{} left the server ({}{} time)",
                user.name,
                user.discriminator,
                times,
                cardinality(times)
            ),
        );
    }
}

fn cardinality(i: u32) -> &'static str {
    match ((i / 10) % 10, i % 10) {
        (1, _) => "th",
        (_, 1) => "st",
        (_, 2) => "nd",
        (_, 3) => "rd",
        (_, _) => "th",
    }
}
