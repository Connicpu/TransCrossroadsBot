use serenity::prelude::*;
use serenity::model::prelude::*;

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
    let result = msg.channel_id.say(format_args!(
        "Okay {}, be extremely careful. Here's the admin destructive action code: {} ({})",
        msg.author.mention(),
        code,
        "@everyone"
    ));
    logres(ctx, result);
}
