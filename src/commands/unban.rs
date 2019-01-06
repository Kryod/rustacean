use serenity::model::prelude::User;
use serenity::framework::standard::ArgError::{ Parse, Eos };

command!(unban(ctx, msg, args) {
    let mut data = ctx.data.lock();
    let db = data.get::<::DbPool>().unwrap();
    let user = args.single::<User>();
    let global = args.single::<bool>();

    let (discord_user, user) = match user {
        Ok(discord_user) => {
            let user = ::models::User::get(&format!("{}", discord_user.id), &db);
            (discord_user, user)
        },
        Err(Parse(e)) => {
            let _ = msg.reply(&format!("Please specify a valid user to unban ({}).", e));
            return Ok(());
        },
        Err(Eos) => {
            let _ = msg.reply("Please specify the user to unban.");
            return Ok(());
        },
    };
    let global = match global {
        Ok(global) => global,
        Err(_) => false,
    };

    let (is_banned, is_banned_globally) = {
        let bans = data.get::<::Bans>().unwrap();
        let is_banned = bans.iter().any(| (user_id, bans_for_user) | {
            *user_id == discord_user.id && bans_for_user.iter().any(| b | {
                b.is_banned_for_guild(msg.guild_id) && !b.is_over()
            })
        });
        let is_banned_globally = bans.iter().any(| (user_id, bans_for_user) | {
            *user_id == discord_user.id && bans_for_user.iter().all(| b | {
                !b.is_over() && b.is_global()
            })
        });
        (is_banned, is_banned_globally)
    };
    if !is_banned {
        let _ = msg.reply("This user is not banned.");
        return Ok(());
    }

    let is_bot_owner = {
        let settings = data.get::<::Settings>().unwrap().lock().unwrap();
        let owners = &settings.bot_owners;
        owners.contains(&msg.author.id)
    };
    if is_banned_globally && !is_bot_owner {
        let _ = msg.reply("You need to be a bot owner to lift a global ban.");
        return Ok(());
    }

    user.unban(msg.guild_id.unwrap(), (is_bot_owner && global) || (is_bot_owner && is_banned_globally), &db);
    let _ = msg.reply(&format!("Successfully unbanned {}!", discord_user));
});
