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

mod commands;

use serenity::framework::StandardFramework;
use serenity::model::event::ResumedEvent;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::http;
use std::collections::{ HashSet, HashMap };
use std::str::FromStr;

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
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

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix("~"))
        .command("ping", |c| c.cmd(commands::meta::ping))
        .command("multiply", |c| c.cmd(commands::math::multiply))
        .command("exec", |c| c.cmd(commands::exec::exec))
        .command("quit", |c| c
            .cmd(commands::owner::quit)
            .owners_only(true)));

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
