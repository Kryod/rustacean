//! Requires the 'framework' feature flag be enabled in your project's
//! `Cargo.toml`.
//!
//! This can be enabled by specifying the feature in the dependency section:
//!
//! ```toml
//! [dependencies.serenity]
//! git = "https://github.com/serenity-rs/serenity.git"
//! features = ["framework", "standard_framework"]
//! ```

#[macro_use] extern crate log;
#[macro_use] extern crate serenity;

extern crate rand;
extern crate simplelog;
extern crate config;
extern crate duct;
extern crate regex;
extern crate typemap;

pub mod commands;
pub mod lang_manager;
mod test;

use lang_manager::LangManager;

use serenity::client::bridge::gateway::{ ShardManager };
use serenity::framework::standard::{ DispatchError, StandardFramework, help_commands};
use serenity::model::event::ResumedEvent;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::{ Client, Context, EventHandler };
use serenity::http;
use std::collections::{ HashSet, HashMap };
use std::str::FromStr;
use std::sync::{ Arc, Mutex };
use typemap::Key;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<serenity::prelude::Mutex<ShardManager>>;
}

struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Settings;

impl Key for Settings {
    type Value = HashMap<String, String>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        info!("Open this link in a web browser to invite {} to a Discord server:\r\nhttps://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=378944", ready.user.name, ready.user.id);

        let ctx = Arc::new(Mutex::new(ctx));
        std::thread::spawn(move || {
            loop {
                set_game_presence_help(&ctx.lock().unwrap());
                std::thread::sleep(std::time::Duration::from_secs(30));

                set_game_presence_languages(&ctx.lock().unwrap());
                std::thread::sleep(std::time::Duration::from_secs(30));

                set_game_presence_exec(&ctx.lock().unwrap());
                std::thread::sleep(std::time::Duration::from_secs(30));
            }
        });

        std::thread::spawn(move || {
            let cleanup_min_age = std::time::Duration::from_secs(60 * 60);
            loop {
                let user_dirs = std::fs::read_dir(commands::exec::get_snippets_directory()).unwrap();
                for user_dir in user_dirs {
                    let snippet_files = std::fs::read_dir(user_dir.unwrap().path()).unwrap();
                    for file in snippet_files {
                        let file = file.unwrap().path();
                        let metadata = std::fs::metadata(&file).unwrap();
                        if metadata.is_file() && metadata.modified().unwrap().elapsed().unwrap() >= cleanup_min_age {
                            let _ = std::fs::remove_file(file);
                        }
                    }
                }

                std::thread::sleep(cleanup_min_age);
            }
        });
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    fn message(&self, _: Context, msg: Message) {
        if msg.content.contains("rust") {
            /*let mut emote : Emoji = Emoji::new();
            emote.id = "509392478491639828";
            msg.react(emote);*/
        }

        if msg.content == "!hello" {
            // The create message builder allows you to easily create embeds and messages
            // using a builder syntax.
            // This example will create a message that says "Hello, World!", with an embed that has
            // a title, description, three fields, and footer.
            if let Err(why) = msg.channel_id.send_message(|m| m
                .content("Hello, World!")
                .embed(|e| e
                    .title("This is a title")
                    .description("This is a description")
                    .fields(vec![
                        ("This is the first field", "This is a field body", true),
                        ("This is the second field", "Both of these fields are inline", true),
                    ])
                    .field("This is the third field", "This is not an inline field", false)
                    .footer(|f| f
                        .text("This is a footer")))) {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

fn init_settings() -> HashMap<String, String> {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("config")).expect("Could not read configuration file")
        .merge(config::Environment::with_prefix("RUST")).unwrap();
    settings.try_into().expect("Could not deserialize configuration")
}

fn init_logging(settings: &HashMap<String, String>) {
    use simplelog::{ CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger };

    let log_level_term = LevelFilter::from_str(settings["log_level_term"].as_ref()).expect("Invalid log level filter");
    let log_level_file = LevelFilter::from_str(settings["log_level_file"].as_ref()).expect("Invalid log level filter");

    let log_file = std::fs::File::create("rustacean.log").expect("Could not create log file");

    CombinedLogger::init(
        vec![
            TermLogger::new(log_level_term, Config::default()).unwrap(),
            WriteLogger::new(log_level_file, Config::default(), log_file),
        ]
    ).unwrap();
}

fn main() {
    let settings = init_settings();
    let command_prefix = settings["command_prefix"].clone();
    init_logging(&settings);

    let mut client = Client::new(&settings["discord_token"], Handler).expect("Err creating client");

    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    let lang_manager = LangManager::new();

    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<Settings>(settings);
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<LangManager>(Arc::new(Mutex::new(lang_manager)));
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix(&command_prefix))
        // Set a function to be called prior to each command execution. This
        // provides the context of the command, the message that was received,
        // and the full name of the command that will be called.
        //
        // You can not use this to determine whether a command should be
        // executed. Instead, `set_check` is provided to give you this
        // functionality.
        .before(|ctx, msg, command_name| {
            debug!("Got command '{}' by user '{}'",
                     command_name,
                     msg.author.name);
            // Increment the number of times this command has been run once. If
            // the command's name does not exist in the counter, add a default
            // value of 0.
            let mut data = ctx.data.lock();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.to_string()).or_insert(0);
            *entry += 1;

            true // if `before` returns false, command processing doesn't happen.
        })
        // Similar to `before`, except will be called directly _after_
        // command execution.
        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => debug!("Processed command '{}'", command_name),
                Err(why) => error!("Command '{}' returned error {:?}", command_name, why),
            }
        })
        // Set a function that's called whenever an attempted command-call's
        // command could not be found.
        .unrecognised_command(|_, msg, unknown_command_name| {
            error!("Could not find command named '{}'", unknown_command_name);
            let _ = msg.channel_id.say(&format!("Could not find command named '{}'", unknown_command_name));
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|_ctx, msg, error| {
            if let DispatchError::RateLimited(seconds) = error {
                let _ = msg.channel_id.say(&format!("Try this again in {} seconds.", seconds));
            }
        })
        .help(help_commands::with_embeds)
        // Time out for exec: Can't be used more than 2 times per 30 seconds, with a 5 second delay
        //.bucket("exec_bucket", 5, 30, 2)
        // Can't be used more than once per 5 seconds:
        .simple_bucket("exec_bucket", 5)
        .command("ping", |c| c.cmd(commands::meta::ping))
        .command("multiply", |c| c.cmd(commands::math::multiply))
        .command("git", |c| c.cmd(commands::git::git))
        .command("exec", |c| c
            .bucket("exec_bucket")
            .cmd(commands::exec::exec))
        .command("languages", |c| c.cmd(commands::languages::languages))
        .command("quit", |c| c
            .cmd(commands::owner::quit)
            .owners_only(true))
        );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

fn get_command_prefix(ctx: &Context) -> String {
    let data = ctx.data.lock();
    let settings = data.get::<Settings>().unwrap();
    settings["command_prefix"].clone()
}

fn set_game_presence_help(ctx: &Context) {
    let prefix = get_command_prefix(ctx);
    set_game_presence(ctx, &format!("Type {}help to get a list of available commands", prefix));
}

fn set_game_presence_languages(ctx: &Context) {
    let prefix = get_command_prefix(ctx);
    set_game_presence(ctx, &format!("Type {}languages to get a list of available languages", prefix));
}

fn set_game_presence_exec(ctx: &Context) {
    let prefix = get_command_prefix(ctx);
    set_game_presence(ctx, &format!("Type {}exec ```language code``` to run a code snippet", prefix));
}

fn set_game_presence(ctx: &Context, game_name: &str) {
    let game = serenity::model::gateway::Game::playing(game_name);
    let status = serenity::model::user::OnlineStatus::Online;
    ctx.set_presence(Some(game), status);
}
