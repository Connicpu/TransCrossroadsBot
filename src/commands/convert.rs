use serenity::model::prelude::*;
use serenity::prelude::*;

use std::collections::HashMap;

use grammar::ast::Command;

static CONVERTIBLE: &str = "testosterone and estradiol";
static RAW_CONVERSION_FACTORS: &[(&str, &[(&str, f64)])] = &[
    (
        "testosterone",
        &[
            ("nmol/l", 0.0347),
            ("ng/ml", 0.01),
            ("ng/dl", 1.0),
            ("ng/100ml", 1.0),
            ("ng/l", 10.0),
            ("ug/l", 0.01),
            ("mcg/l", 0.01),
        ],
    ),
    (
        "estradiol",
        &[
            ("pmol/l", 3.6713),
            ("pg/ml", 1.0),
            ("pg/dl", 100.0),
            ("pg/100ml", 100.0),
            ("pg/l", 1000.0),
            ("ng/l", 1.0),
        ],
    ),
];

lazy_static! {
    static ref CONVERSION_FACTORS: HashMap<String, HashMap<String, f64>> = RAW_CONVERSION_FACTORS
        .iter()
        .map(|&(s, h)| (
            s.to_string(),
            h.iter().map(|&(s, v)| (s.to_string(), v)).collect()
        ))
        .collect();
}

pub fn convert(_ctx: &Context, msg: &Message, cmd: &Command) {
    let (value, chem, from, to) = match cmd {
        Command::Convert {
            value,
            chemical,
            from,
            to,
        } => (value, chemical, from, to),
        _ => return,
    };

    let chemlist = match CONVERSION_FACTORS.get(chem) {
        Some(list) => list,
        None => {
            let _ = msg.reply(&format!(
                "I'm sorry, I don't know the compound `{}`. I can convert {}",
                chem, CONVERTIBLE,
            ));
            return;
        }
    };

    let fromfactor = match chemlist.get(from) {
        Some(&factor) => factor,
        None => {
            let available = chemlist
                .keys()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let _ = msg.reply(&format!(
                "I'm sorry, I don't know how to convert with `{}` for `{}`. I can do: {}",
                from, chem, available
            ));
            return;
        }
    };

    let tofactor = match chemlist.get(to) {
        Some(&factor) => factor,
        None => {
            let available = chemlist
                .keys()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let _ = msg.reply(&format!(
                "I'm sorry, I don't know how to convert with `{}` for `{}`. I can do: {}",
                to, chem, available
            ));
            return;
        }
    };

    let newvalue = value * (tofactor / fromfactor);
    let _ = msg.reply(&format!(
        "{} {} of {} is {:.3} {}",
        value, from, chem, newvalue, to,
    ));
}
