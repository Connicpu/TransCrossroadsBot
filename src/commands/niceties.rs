use serenity::model::prelude::*;

pub fn thank_you(msg: &Message) {
    let _ = msg.react("\u{2764}");
}
