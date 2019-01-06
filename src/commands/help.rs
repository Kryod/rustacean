use std::sync::Arc;
use std::collections::HashMap;

use serenity::prelude::Context;
use serenity::model::prelude::Message;
use serenity::framework::standard::{ Args, HelpOptions, CommandGroup, help_commands };

pub fn help(
    _ctx: &mut Context,
    msg: &Message,
    help_options: &HelpOptions,
    groups: HashMap<String, Arc<CommandGroup>>,
    args: &Args,
) -> Result<(), serenity::framework::standard::CommandError> {
    // WIP...

    //let formatted = help_commands::create_customised_help_data(&groups, args, help_options, msg);
    help_commands::with_embeds(_ctx, msg, help_options, groups, args)
    //Ok(())
}
