use crate::{ models, DbPool, Bans, Settings };

use serenity::{
    prelude::Context,
    model::{
        prelude::UserId,
        channel::Message,
    },
    framework::standard::{
        Args, CommandResult,
        macros::command, ArgError::Parse,
    },
};

#[command]
#[description = "Lifts a previously issued ban. This command will not unban the target user from the Discord server, however."]
#[example = "@user"]
#[required_permissions("ADMINISTRATOR")]
#[only_in(guilds)]
#[owner_privilege]
fn unban(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let db = data.get::<DbPool>().unwrap();
    let user_id = args.single::<UserId>();
    let global = args.single::<bool>();

    let (discord_user, user) = match user_id {
        Ok(user_id) => {
            let discord_user = user_id.to_user(&ctx).unwrap();
            let user = models::User::get(user_id, &db);
            (discord_user, user)
        },
        Err(Parse(e)) => {
            let _ = msg.reply(&ctx, &format!("Please specify a valid user to unban ({}).", e))?;
            return Ok(());
        },
        Err(_e) => {
            let _ = msg.reply(&ctx, "Please specify the user to unban.")?;
            return Ok(());
        },
    };
    let global = match global {
        Ok(global) => global,
        Err(_) => false,
    };

    let (is_banned, is_banned_globally) = {
        let bans = data.get::<Bans>().unwrap();
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
        let _ = msg.reply(&ctx, "This user is not banned.")?;
        return Ok(());
    }

    let is_bot_owner = {
        let settings = data.get::<Settings>().unwrap().lock().unwrap();
        let owners = &settings.bot_owners;
        owners.contains(&msg.author.id)
    };
    if is_banned_globally && !is_bot_owner {
        let _ = msg.reply(&ctx, "You need to be a bot owner to lift a global ban.")?;
        return Ok(());
    }

    let lifted_ban_id = user.unban(msg.guild_id.unwrap(), (is_bot_owner && global) || (is_bot_owner && is_banned_globally), &db);
    match lifted_ban_id {
        Some(id) => {
            let bans = data.get_mut::<Bans>().unwrap();
            match bans.get_mut(&discord_user.id) {
                Some(bans) => bans.retain(|ban| ban.get_id() != id),
                None => {},
            };
            msg.reply(&ctx, &format!("Successfully unbanned {}!", discord_user))?
        },
        None => msg.reply(&ctx, &format!("Could not find ban entry for {} in database.", discord_user))?,
    };

    Ok(())
}
