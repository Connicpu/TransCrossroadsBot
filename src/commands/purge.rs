use chrono::{DateTime, Duration, Utc};
use serenity::Error;
use serenity::model::prelude::*;
use serenity::prelude::*;

use std::fmt;
use std::str;
use std::thread;
use std::time::{self, SystemTime};

use grammar::ast::Command;
use {logres, staff_alert, state};

pub fn issue_code(ctx: &Context, msg: &Message, _cmd: &Command) {
    let state = state(ctx);
    let staff_alert = staff_alert(ctx);

    if msg.channel_id != staff_alert.admin_channel {
        logres(ctx, msg.delete());
        logres(
            ctx,
            staff_alert.admin_channel.say(format_args!(
                "Psst, {}, do that in here ya goof",
                msg.author.mention()
            )),
        );
        return;
    }

    let code = state.challenge_code.issue();
    println!("Issue code: {}", code);
    let result = msg.channel_id.say(format_args!(
        "Okay {}, be extremely careful. Here's the admin destructive action code: {} ({})",
        msg.author.mention(),
        code,
        "@everyone"
    ));
    logres(ctx, result);
}

pub fn purge_channel(ctx: &Context, msg: &Message, cmd: &Command) {
    let (channel, time, time2, code) = match cmd {
        Command::PurgeChannel(channel, time, time2, code) => {
            (*channel, *time, *time2, code.clone())
        }
        _ => return,
    };

    let state = state(ctx);
    let staff_alert = staff_alert(ctx);

    if msg.channel_id != staff_alert.admin_channel {
        logres(ctx, msg.delete());
        logres(
            ctx,
            staff_alert.admin_channel.say(format_args!(
                "Psst, {}, do that in here ya goof",
                msg.author.mention()
            )),
        );
        return;
    }

    let real_code = state.challenge_code.take();
    if Some(code.trim()) != real_code.as_ref().map(|s| s.trim()) {
        println!("{:?} != {:?}", code, real_code);
        let _ = msg.reply("Invalid challenge code. Start over.");
        return;
    }

    let ctx = ctx.clone();
    let msg = msg.clone();
    thread::spawn(move || {
        let mut messages = Vec::with_capacity(256);

        let get_msgs = |messages: &mut Vec<MessageId>| -> Result<(), Error> {
            let mut filtered = false;
            let mut msgs = channel.messages(|get| get.most_recent().limit(100))?;
            msgs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            let mut i = 0;
            loop {
                messages.extend(
                    msgs.iter()
                        .rev()
                        .filter(|msg| {
                            if msg.timestamp.timestamp() < time.timestamp() {
                                filtered = true;
                                false
                            } else {
                                true
                            }
                        })
                        .filter(|msg| msg.timestamp.timestamp() < time2.timestamp())
                        .filter(|msg| !msg.pinned)
                        .map(|m| m.id),
                );

                if filtered || msgs.len() < 100 {
                    break;
                }

                i += 1;
                if i == 1 {
                    let _ = msg.reply("I'm gathering a lot of messages. This could take a while depending on the range you asked for.");
                }
                if i % 25 == 0 {
                    let status = format!("Large delete gather progress: {}", messages.len());
                    let _ = msg.reply(&status);
                    println!("{}", status);
                }

                let first = msgs[0].id;
                thread::sleep(time::Duration::from_millis(100));
                msgs = channel.messages(|get| get.before(first).limit(100))?;
                msgs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            }

            Ok(())
        };

        match get_msgs(&mut messages) {
            Ok(()) => (),
            r => {
                logres(&ctx, r);
                return;
            }
        };

        let now: DateTime<Utc> = SystemTime::now().into();
        let time_diff = now.timestamp() - time.timestamp();
        let time_diff = Duration::seconds(time_diff);
        let time_diff2 = now.timestamp() - time2.timestamp();
        let time_diff2 = Duration::seconds(time_diff2);

        let count = messages.len();
        let reply = format!(
            "Okay, you're about to delete {} messages from {} after {} ({} ago) and before {} ({} ago). Please confirm.",
            count,
            channel.mention(),
            time.to_rfc2822(),
            DurationFmt(time_diff),
            time2.to_rfc2822(),
            DurationFmt(time_diff2),
        );

        let _ = msg.reply(&reply);

        state.challenge_code.set_purge(Some((channel, messages)));
    });
}

fn header_int(data: Option<&[Vec<u8>]>) -> Option<u64> {
    data.and_then(|parts| parts.get(0))
        .and_then(|part| str::from_utf8(part).ok())
        .and_then(|part| part.parse().ok())
}

pub fn execute_purge(ctx: &Context, msg: &Message, cmd: &Command) {
    let num = match cmd {
        Command::ExecutePurge(num) => *num,
        _ => return,
    };

    let state = state(ctx);
    let staff_alert = staff_alert(ctx);

    if msg.channel_id != staff_alert.admin_channel {
        logres(ctx, msg.delete());
        logres(
            ctx,
            staff_alert.admin_channel.say(format_args!(
                "Psst, {}, do that in here ya goof",
                msg.author.mention()
            )),
        );
        return;
    }

    let (channel, messages) = match state.challenge_code.take_purge() {
        Some((channel, messages)) => (channel, messages),
        _ => {
            logres(ctx, msg.reply("You didn't set it up. Start over!"));
            return;
        }
    };

    if num as usize != messages.len() {
        let _ = msg.reply("Wrong number of messages. Start over!");
        return;
    }

    let ctx = ctx.clone();
    let msg = msg.clone();
    thread::spawn(move || {
        let mut i = 0;
        let mut j_i = -1isize as usize;
        for (j, chunk) in messages.chunks(100).enumerate() {
            loop {
                let res = channel.delete_messages(chunk);
                match res {
                    Ok(_) => (),
                    Err(Error::Http(HttpError::UnsuccessfulRequest(ref resp)))
                        if header_int(resp.headers.get_raw("X-RateLimit-Remaining")) == Some(0)
                            && j_i != j =>
                    {
                        let now = time::SystemTime::now();
                        let reset = header_int(resp.headers.get_raw("X-RateLimit-Reset"))
                            .map(|reset| time::UNIX_EPOCH + time::Duration::from_secs(reset))
                            .unwrap_or(now.clone());
                        let wait = reset
                            .duration_since(now)
                            .unwrap_or(time::Duration::from_secs(0))
                            + time::Duration::from_secs(5);
                        let _ = msg.reply(&format!(
                            "Oops, hit a rate limit! Resets at {}. Waiting for {}",
                            DateTime::<Utc>::from(reset).to_rfc2822(),
                            DurationFmt(Duration::from_std(wait).unwrap_or(Duration::seconds(0))),
                        ));
                        i = 9;
                        j_i = j;
                        thread::sleep(wait);
                        continue;
                    }
                    err => {
                        thread::sleep(time::Duration::from_millis(200));
                        logres(&ctx, err);
                        thread::sleep(time::Duration::from_millis(200));
                        let _ = msg.reply("Something went wrong");
                        return;
                    }
                }
                thread::sleep(time::Duration::from_millis(100));
                break;
            }

            i += 1;
            if i % 10 == 0 {
                let _ = msg.reply(&format!("{} messages deleted...", j * 100));
            }
        }

        logres(&ctx, msg.reply("DONE!~ *Phew*"));
    });
}

pub fn cancel_purge(ctx: &Context) {
    let state = state(ctx);
    state.challenge_code.take();
    state.challenge_code.take_purge();
}

struct DurationFmt(Duration);

impl fmt::Display for DurationFmt {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let d = self.0;
        if d.num_weeks() != 0 {
            write!(fmt, "{} weeks, ", d.num_weeks())?;
        }
        if d.num_days() % 7 != 0 {
            write!(fmt, "{} days, ", d.num_days() % 7)?;
        }
        if d.num_hours() % 24 != 0 {
            write!(fmt, "{} hours, ", d.num_hours() % 24)?;
        }
        if d.num_minutes() % 60 != 0 {
            write!(fmt, "{} minutes, ", d.num_minutes() % 60)?;
        }

        write!(fmt, "{} seconds", d.num_seconds() % 60)?;

        Ok(())
    }
}
