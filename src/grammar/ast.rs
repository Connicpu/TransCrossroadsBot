use serenity::prelude::*;
use serenity::model::prelude::*;

#[derive(Debug)]
pub enum Command {
    HowManyPosts(UserId),
    SetPronouns {
        target: UserId,
        pronouns: Vec<String>,
    },

    GiveRoles {
        target: UserId,
        roles: Vec<String>,
    },
    TakeRoles {
        target: UserId,
        roles: Vec<String>,
    },

    RescanPronouns(String, String),
    RescanRoles(String, String),
    AliasRole(String, String),
    RemoveAlias(String),

    ChallengeCode,
    PurgeChannel(ChannelId, String),

    ListAllRoles,
    ListAllAliases,

    ThankYou,
    OmeaWaNoShinderu,
}

fn has_perm(member: &Member, perm: Permissions) -> bool {
    member
        .permissions()
        .map(|p| p & perm == perm)
        .unwrap_or(false)
}

impl Command {
    pub fn is_authorized(&self, cmduser: UserId, member: &Member) -> bool {
        use self::Command::*;
        if member.roles.is_empty() {
            return false;
        }

        if has_perm(member, Permissions::ADMINISTRATOR) {
            return true;
        }

        match self {
            HowManyPosts(target) => *target == cmduser,
            SetPronouns { target, .. } => {
                *target == cmduser || has_perm(member, Permissions::MANAGE_ROLES)
            }
            GiveRoles { target, .. } => {
                *target == cmduser || has_perm(member, Permissions::MANAGE_ROLES)
            }
            TakeRoles { target, .. } => {
                *target == cmduser || has_perm(member, Permissions::MANAGE_ROLES)
            }
            ListAllRoles | ListAllAliases => true,
            ThankYou => true,
            OmeaWaNoShinderu => true,
            _ => false,
        }
    }

    pub fn execute(&mut self, ctx: &Context, msg: &Message) {
        use self::Command::*;
        use commands;
        match self {
            SetPronouns { .. } => {
                commands::pronouns::set_pronouns(ctx, msg, self);
            }
            RescanPronouns { .. } => {
                commands::pronouns::scan_pronouns(ctx, msg, self);
            }
            RescanRoles { .. } => {
                commands::roles::scan_roles(ctx, msg, self);
            }
            AliasRole { .. } => {
                commands::roles::alias_role(ctx, msg, self);
            }
            RemoveAlias { .. } => {
                commands::roles::remove_alias(ctx, msg, self);
            }
            ListAllRoles => {
                commands::roles::list_roles(ctx, msg, self);
            }
            ListAllAliases => {
                commands::roles::list_aliases(ctx, msg, self);
            }
            GiveRoles { .. } => {
                commands::roles::give_roles(ctx, msg, self);
            }
            TakeRoles { .. } => {
                commands::roles::take_roles(ctx, msg, self);
            }
            ThankYou => {
                commands::niceties::thank_you(msg);
            }
            OmeaWaNoShinderu => {
                commands::niceties::omea_wa_no_shinderu(msg);
            }
            _ => {
                let _ = msg.reply("I'm sorry, I don't know how to do that yet :<");
            }
        }
    }

    pub fn pronouns<I, T>(target: UserId, pronouns: I) -> Command
    where
        I: IntoIterator<Item = T>,
        T: TagToStr,
    {
        let pronouns = pronouns.into_iter().map(|i| i.to_str()).collect();
        Command::SetPronouns { target, pronouns }
    }

    pub fn give_roles<I, T>(target: UserId, roles: I) -> Command
    where
        I: IntoIterator<Item = T>,
        T: TagToStr,
    {
        let roles = roles.into_iter().map(|i| i.to_str()).collect();
        Command::GiveRoles { target, roles }
    }

    pub fn take_roles<I, T>(target: UserId, roles: I) -> Command
    where
        I: IntoIterator<Item = T>,
        T: TagToStr,
    {
        let roles = roles.into_iter().map(|i| i.to_str()).collect();
        Command::TakeRoles { target, roles }
    }
}

pub trait TagToStr {
    fn to_str(self) -> String;
}

impl TagToStr for String {
    fn to_str(self) -> String {
        self
    }
}

impl<'a> TagToStr for &'a str {
    fn to_str(self) -> String {
        self.to_string()
    }
}

impl<'a> TagToStr for &'a &'a str {
    fn to_str(self) -> String {
        self.to_string()
    }
}

impl<'a> TagToStr for &'a String {
    fn to_str(self) -> String {
        self.clone()
    }
}
