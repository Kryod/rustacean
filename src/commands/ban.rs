use std::collections::hash_map::Entry::{ Vacant, Occupied };

use serenity::framework::standard::ArgError::{ Eos, Parse };
use serenity::model::prelude::{ User, Permissions };
use chrono::prelude::{ NaiveDate, NaiveDateTime };

command!(ban(ctx, msg, args) {
    let mut data = ctx.data.lock();

    let (discord_user, new_ban) = {
        let db = data.get::<::DbPool>().unwrap();

        let user = args.single::<User>();
        let time = args.single::<String>();
        let global = args.single::<bool>();

        let (discord_user, user) = match user {
            Ok(discord_user) => {
                let user = ::models::User::get(discord_user.id, &db);
                (discord_user, user)
            },
            Err(Parse(e)) => {
                let _ = msg.reply(&format!("Please specify a valid user to ban ({}).", e));
                return Ok(());
            },
            Err(Eos) => {
                let _ = msg.reply("Please specify the user to ban.");
                return Ok(());
            },
        };

        let (is_bot_owner, is_target_owner) = {
            let settings = data.get::<::Settings>().unwrap().lock().unwrap();
            let owners = &settings.bot_owners;
            (
                owners.contains(&msg.author.id),
                owners.contains(&discord_user.id),
            )
        };

        if discord_user.id == msg.author.id {
            let _ = msg.reply("You cannot ban yourself...");
            return Ok(());
        }

        if is_target_owner {
            let _ = msg.reply("You cannot ban this user.");
            return Ok(());
        }

        let channel = msg.channel_id;
        let channel = channel.to_channel();
        let channel = match channel {
            Ok(channel) => channel,
            Err(e) => {
                error!("ban.rs: Could not fetch channel: {}", e);
                let _ = msg.reply(&format!("An error occurred ({})", e));
                return Ok(());
            },
        };

        let is_target_admin = match channel.guild() {
            Some(guild_channel_lock) => {
                let guild = guild_channel_lock.read().guild();
                match guild {
                    Some(guild_lock) => {
                        let permissions = guild_lock.read().member_permissions(discord_user.id);
                        permissions.contains(Permissions::ADMINISTRATOR)
                    },
                    None => false,
                }
            },
            None => false,
        };

        if !is_bot_owner && is_target_admin {
            let _ = msg.reply("You cannot ban an other guild administrator.");
            return Ok(());
        }

        let guild = match global {
            Ok(global) => {
                match global {
                    true => {
                        if !is_bot_owner {
                            let _ = msg.reply("You need to be a bot owner to ban someone globally.");
                            return Ok(());
                        }
                        None
                    },
                    false => msg.guild_id,
                }
            },
            Err(_) => msg.guild_id,
        };
        let is_already_banned = {
            let bans = data.get::<::Bans>().unwrap();
            bans.iter().any(| (user_id, bans_for_user) | {
                *user_id == discord_user.id && bans_for_user.iter().any(| b | {
                    b.is_banned_for_guild(msg.guild_id) && !b.is_over()
                })
            })
        };
        if is_already_banned {
            let _ = msg.reply("This user is already banned.");
            return Ok(());
        }

        let time = match time {
            Ok(mut time) => {
                time = time.trim().into();
                time = time.to_lowercase();
                if time == "permanent" || time == "infinite" || time == "forever" {
                    None
                } else {
                    match NaiveDateTime::parse_from_str(&time, "%Y-%m-%d-%H:%M") {
                        Ok(time) => Some(time),
                        Err(_) => {
                            match NaiveDate::parse_from_str(&time, "%Y-%m-%d") {
                                Ok(date) => Some(date.and_hms(0, 0, 0)),
                                Err(_) => {
                                    let _ = msg.reply("Invalid ban end time. Please use the format \"yyyy-mm-dd[-hh:mm]\"");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            },
            Err(_) => None,
        };

        let new_ban = user.ban(&db, time, guild);
        (discord_user, new_ban)
    };

    let mut bans = data.get_mut::<::Bans>().unwrap();
    let vec = match bans.entry(discord_user.id) {
        Vacant(entry) => entry.insert(Vec::new()),
        Occupied(entry) => entry.into_mut(),
    };
    vec.push(new_ban);

    let _ = msg.reply(&format!("<:banhammer:525343781441110017> Banned {}!", discord_user));
});
