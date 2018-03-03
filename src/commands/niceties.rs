use serenity::model::prelude::*;

pub fn thank_you(msg: &Message) {
    let _ = msg.react("\u{2764}");
}

pub fn omea_wa_no_shinderu(msg: &Message) {
    let _ = msg.react("\u{1F632}");
}
