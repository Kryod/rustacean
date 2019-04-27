#[macro_use] extern crate log;
#[macro_use] extern crate diesel;
#[macro_use] extern crate serenity;
#[macro_use] extern crate serde_derive;

extern crate rand;
extern crate simplelog;
extern crate toml;
extern crate serde;
extern crate duct;
extern crate regex;
extern crate typemap;
extern crate chrono;
extern crate reqwest;

pub mod commands;
pub mod lang_manager;
pub mod tools;
pub mod schema;
pub mod models;
pub mod dbl;
mod test;

use lang_manager::LangManager;

use serenity::client::bridge::gateway::{ ShardManager };
use serenity::framework::standard::{ DispatchError, StandardFramework };
use serenity::model::prelude::{ Ready, Message, ResumedEvent, UserId };
use serenity::prelude::{ Client, Context, EventHandler };
use serenity::model::permissions::Permissions;
use serenity::http;
use serenity::model::channel::Embed;
use diesel::SqliteConnection;
use diesel::r2d2::{ ConnectionManager, Pool };
use typemap::Key;
use std::io::{ Error, ErrorKind };

use std::io::Read;
use std::collections::{ HashSet, HashMap };
use std::str::FromStr;
use std::sync::{ Arc, Mutex };
use std::process::Command;

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<serenity::prelude::Mutex<ShardManager>>;
}

#[derive(Deserialize, Clone)]
struct Settings {
    pub discord_token: String,
    pub dbl_api_key: Option<String>,
    pub command_prefix: String,
    pub log_level_term: String,
    pub log_level_file: String,
    pub db_connection_pool_size: u32,
    pub bot_owners: Vec<serenity::model::prelude::UserId>,
    pub webhook_id: Option<u64>,
    pub webhook_token: Option<String>,
    pub webhook_frequency: Option<u64>,
    pub webhook_role: Option<String>,
    pub cpu_load: String,
    pub ram_load: String
}

impl Key for Settings {
    type Value = Arc<Mutex<Settings>>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        let ctx = Arc::new(Mutex::new(ctx));

        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            match shard[0] {
                0 => {
                    info!("Connected as {}", ready.user.name);
                    info!("Open this link in a web browser to invite {} to a Discord server:\r\nhttps://discordapp.com/oauth2/authorize?client_id={}&scope=bot&permissions=378944", ready.user.name, ready.user.id);
                }
                1 => presence_status_thread(ready.user.id, ctx),
                2 => cargo_test_thread(ctx),
                3 => snippets_cleanup_thread(),
                _ => { },
            };

            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name,
                shard[0],
                shard[1],
            );
        }
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }

    fn message(&self, _: Context, _msg: Message) {

    }
}

fn presence_status_thread(user_id: UserId, ctx: Arc<Mutex<Context>>) {
    let dbl_api_key = {
        let ctx_lock = ctx.lock().unwrap();
        let data = ctx_lock.data.lock();
        let settings = data.get::<Settings>().unwrap().lock().unwrap();
        settings.dbl_api_key.clone()
    };

    std::thread::spawn(move || {
        // Game presence status rotation
        loop {
            set_game_presence_help(&ctx.lock().unwrap());
            std::thread::sleep(std::time::Duration::from_secs(30));

            set_game_presence_languages(&ctx.lock().unwrap());
            std::thread::sleep(std::time::Duration::from_secs(30));

            set_game_presence_exec(&ctx.lock().unwrap());
            std::thread::sleep(std::time::Duration::from_secs(30));

            let guilds = get_guilds();
            match guilds {
                Ok(count) => {
                    set_game_presence(&ctx.lock().unwrap(), &format!("On {} servers", count));
                    if dbl_api_key.is_some() {
                        let _ = dbl::post_stats(user_id, dbl_api_key.as_ref().unwrap(), count);
                    }
                    std::thread::sleep(std::time::Duration::from_secs(15));
                },
                Err(e) => error!("Error while retrieving guild count: {}", e),
            };
        }
    });
}

fn cargo_test_thread(ctx: Arc<Mutex<Context>>) {
    let (webhook_id, webhook_token, webhook_freq, webhook_role) = {
        let ctx_lock = ctx.lock().unwrap();
        let data = ctx_lock.data.lock();
        let settings = data.get::<Settings>().unwrap().lock().unwrap();
        (
            settings.webhook_id.clone(),
            settings.webhook_token.clone(),
            settings.webhook_frequency.clone(),
            settings.webhook_role.clone(),
        )
    };

    std::thread::spawn(move || {
        // Periodic tests to check if bot is broken
        if webhook_id.is_none() || webhook_token.is_none() || webhook_freq.is_none() || webhook_role.is_none() {
            return;
        }
        let (webhook_id, webhook_token, webhook_freq, webhook_role) = (
            webhook_id.unwrap(),
            webhook_token.unwrap(),
            webhook_freq.unwrap(),
            webhook_role.unwrap(),
        );

        let test_freq = std::time::Duration::from_secs(60 * webhook_freq);

        loop {
            info!("Running test command!");

            let webhook = http::get_webhook_with_token(webhook_id, &webhook_token)
                .expect("valid webhook");

            let mut cargo = Command::new("cargo");
            let cargo_test = match cfg!(debug_assertions) {
                true => cargo.arg("test"),
                false => cargo.arg("test").arg("--release"),
            };
            let output = cargo_test.output();
            let output = match output {
                Ok(out) => out,
                Err(err) => {
                    error!("Could not run test: {}", err);
                    let embed = Embed::fake(|e| e
                        .title("Rustacean encountered an error")
                        .colour(serenity::utils::Colour::RED)
                        .description("Could not run test")
                        .field("Error", err, true));
                    let _ = webhook.execute(false, |w| w
                        .content(&format!("<@&{}>, we have a problem!", webhook_role))
                        .username("Rustacean Alert")
                        .embeds(vec![embed]))
                        .expect("Error executing");
                    break;
                }
            };

            info!("Ran test command!");

            let mut stdout = ::std::str::from_utf8(&output.stdout)
                .map_err(| e | Error::new(ErrorKind::InvalidData, e))
                .unwrap()
                .to_owned();
            let mut stderr = ::std::str::from_utf8(&output.stderr)
                .map_err(| e | Error::new(ErrorKind::InvalidData, e))
                .unwrap()
                .to_owned();

            stdout.truncate(2000);
            stderr.truncate(1000);

            let exit_code = output.status.code();
            let (embed, ping) = match exit_code {
                Some(0) => {
                    info!("Tests passed successfully!");
                    (Embed::fake(|e| e
                        .title("Rustacean is doing fine")
                        .colour(serenity::utils::Colour::DARK_GREEN)
                        .description(&stdout)), false)

                },
                Some(_) => {
                    warn!("An error occured!");
                    (Embed::fake(|e| e
                        .title("Rustacean encountered an issue")
                        .colour(serenity::utils::Colour::RED)
                        .description(&stdout)
                        .field("Error", &stderr, true)), true)
                },
                None => {
                    error!("An error occured!");
                    (Embed::fake(|e| e
                        .title("Rustacean encountered an error")
                        .colour(serenity::utils::Colour::RED)
                        .description(&stdout)
                        .field("Error", &stderr, true)), true)
                },
            };

            let content = match ping {
                true => format!("<@&{}>, we have a problem!", webhook_role),
                false => "Everything is fine :sunny:".into()
            };

            let _ = webhook.execute(false, |w| w
                        .content(&content)
                        .username("Rustacean Alert")
                        .embeds(vec![embed]))
                        .expect("Error executing");

            std::thread::sleep(test_freq);
        }
    });
}

fn snippets_cleanup_thread() {
    std::thread::spawn(move || {
        // Periodic snippets directory cleanup
        let cleanup_min_age = std::time::Duration::from_secs(60 * 60);
        loop {
            match commands::exec::get_snippets_directory() {
                Ok(snippets_dir) => {
                    let user_dirs = std::fs::read_dir(snippets_dir).unwrap();
                    for user_dir in user_dirs {
                        let snippet_files = std::fs::read_dir(user_dir.unwrap().path()).unwrap();
                        for file in snippet_files {
                            let file = file.unwrap().path();
                            let metadata = std::fs::metadata(&file).unwrap();
                            if let Ok(date) =  metadata.created() {
                                if metadata.is_file() && date.elapsed().unwrap() >= cleanup_min_age {
                                    let _ = std::fs::remove_file(file);
                                }
                            }
                        }
                    }
                },
                Err(_) => {},
            };

            std::thread::sleep(cleanup_min_age);
        }
    });
}

pub type DbPoolType = Arc<Pool<ConnectionManager<SqliteConnection>>>;
pub struct DbPool(DbPoolType);

impl Key for DbPool {
    type Value = DbPoolType;
}

struct Bans;
impl Key for Bans {
    type Value = HashMap<serenity::model::prelude::UserId, Vec<models::Ban>>;
}

fn get_guilds() -> Result<usize, serenity::Error> {
    let mut count = 0;
    let mut last_guild_id = 0;
    loop {
        let guilds = serenity::http::get_guilds(&serenity::http::GuildPagination::After(last_guild_id.into()), 100)?;
        let len = guilds.len();
        count += len;
        if len < 100 {
            break;
        }
        match guilds.last() {
            Some(last) => last_guild_id = *last.id.as_u64(),
            None => {}
        };
    }

    Ok(count)
}

fn init_settings() -> Settings {
    let mut f = std::fs::File::open("config.toml").expect("Could not find the config.toml file. Please copy config.toml.example to config.toml and edit the resulting file");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("Could not read configuration file");
    toml::from_str(&contents).expect("Could not deserialize configuration")
}

fn init_logging(settings: &Settings) {
    use simplelog::{ CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger };

    let mut config = Config::default();
    config.time_format = Some("[%Y-%m-%d %H:%M:%S]");

    let log_level_term = LevelFilter::from_str(settings.log_level_term.as_ref()).expect("Invalid log level filter");
    let log_level_file = LevelFilter::from_str(settings.log_level_file.as_ref()).expect("Invalid log level filter");

    let log_file = std::fs::File::create("rustacean.log").expect("Could not create log file");

    CombinedLogger::init(
        vec![
            TermLogger::new(log_level_term, config).unwrap(),
            WriteLogger::new(log_level_file, config, log_file),
        ]
    ).unwrap();
}

fn main() {
    if tools::tools() {
        return;
    }

    if !std::path::PathBuf::from("rustacean.sqlite3").exists() {
        tools::update_db::update_db();
    }

    let settings = init_settings();
    let command_prefix = settings.command_prefix.clone();
    init_logging(&settings);

    let mut client = Client::new(&settings.discord_token, Handler).expect("Err creating client");

    let manager: ConnectionManager<SqliteConnection> = ConnectionManager::new("rustacean.sqlite3");
    let pool = Pool::builder()
        .max_size(settings.db_connection_pool_size)
        .build(manager)
        .expect("Could not build database connection pool.");
    let pool = Arc::new(pool);

    let owners = match http::get_current_application_info() {
        Ok(info) => {
            let mut set = HashSet::new();
            set.insert(info.owner.id);

            set
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    models::Ban::cleanup_outdated_bans(&pool);

    let mut lang_manager = LangManager::new();
    lang_manager.check_available_languages();

    {
        let mut data = client.data.lock();
        data.insert::<Settings>(Arc::new(Mutex::new(settings)));
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<LangManager>(Arc::new(Mutex::new(lang_manager)));
        data.insert::<DbPool>(pool.clone());
        data.insert::<Bans>(models::Ban::get_bans(&pool));
    }

    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .owners(owners)
            .prefix(&command_prefix))
        .before(| ctx, msg, _cmd_name | {
            let data = ctx.data.lock();
            let bans = data.get::<Bans>().unwrap();
            match bans.get(&msg.author.id) {
                Some(bans) => {
                    let banned = bans.iter().any(| ban | {
                        ban.is_banned_for_guild(msg.guild_id)
                    });
                    if banned {
                        let _ = msg.reply("You cannot run commands while being banned.");
                    }
                    !banned
                },
                None => true,
            }
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|_ctx, msg, error| {
            match error {
                DispatchError::RateLimited(seconds) => {
                    let _ = msg.reply(&format!("Try this again in {} seconds.", seconds));
                },
                DispatchError::OnlyForOwners | DispatchError::LackingRole | DispatchError::BlockedUser | DispatchError::LackOfPermissions(_) => {
                    let _ = msg.reply("You are not allowed to do this.");
                },
                DispatchError::BlockedGuild => {
                    let _ = msg.reply("Rustacean is not available on this server because its owner has been banned.");
                },
                _ => {},
            };
        })
        .help(serenity::framework::standard::help_commands::with_embeds)
        // Time out for exec: Can't be used more than 2 times per 30 seconds, with a 5 second delay
        //.bucket("exec_bucket", 5, 30, 2)
        // Can't be used more than once per 5 seconds:
        .simple_bucket("exec_bucket", 5)
        .group(":desktop: Basic", |g| g
            .command("git", |c| c
                .cmd(commands::git::git)
                .batch_known_as(["github", "repository", "repo"].iter())
                .desc("Get a link to Rustacean's GitHub repository."))
            .command("support", |c| c
                .cmd(commands::support::support)
                .desc("Get a link to Rustacean's support Discord server."))
            .command("invite", |c| c
                .cmd(commands::invite::invite)
                .desc("Get an invite link to add Rustacean to other servers."))
            .command("exec", |c| c
                .cmd(commands::exec::exec)
                .after(|_ctx: &mut Context, msg: &Message, _res: &Result<(), serenity::framework::standard::CommandError>| {
                    let _ = commands::exec::cleanup_user_snippet_directory(msg.author.id);
                })
                .batch_known_as(["execute", "run", "code"].iter())
                .desc(&format!("Executes a code snippet. Your message needs to look like this:\r\n{}exec\r\n\\`\\`\\`language\r\n\r\ncode...\r\n\\`\\`\\`\r\nwhere `language` is the language of your choice.\r\nFor example:\r\n{}exec\r\n\\`\\`\\`javascript\r\nconsole.log(\"hi!\");\r\n\\`\\`\\`", command_prefix, command_prefix))
                .bucket("exec_bucket"))
            .command("languages", |c| c
                .cmd(commands::languages::languages)
                .batch_known_as(["langs", "language", "lang"].iter())
                .desc(&format!("Get a list of available programming languages for the `{}exec` command.", command_prefix)))
            .command("versions", |c| c
                .cmd(commands::versions::versions)
                .batch_known_as(["version", "versions", "ver"].iter())
                .desc(&format!("Get a list of versions for the available programming languages for the `{}exec` command.", command_prefix)))
            .command("stats", |c| c
                .cmd(commands::stats::stats)
                .batch_known_as(["stats", "stat"].iter())
                .desc(&format!("Gets statistics about usage of languages for the `{}exec` command.", command_prefix)))
        )
        .group(":star: Administrator", |g| g
            .command("ban", |c| c
                .cmd(commands::ban::ban)
                .desc("Ban a user from using the bot. This command will not ban the target user from the Discord server, however.")
                .example("@user 2019-11-24")
                .guild_only(true)
                .required_permissions(Permissions::ADMINISTRATOR)
                .owner_privileges(true))
            .command("unban", |c| c
                .cmd(commands::unban::unban)
                .desc("Lifts a previously issued ban. This command will not unban the target user from the Discord server, however.")
                .example("@user")
                .guild_only(true)
                .required_permissions(Permissions::ADMINISTRATOR)
                .owner_privileges(true))
        )
        .group(":robot: Bot owner", |g| g
            .command("quit", |c| c
                .cmd(commands::owner::quit)
                .owners_only(true))
            .command("logs", |c| c
                .cmd(commands::logs::logs)
                .desc("Returns logs of the bot. You can specify a string to search (INFO, DEBUG, ...). By default it gives the last 11 lignes.")
                .example("20 INFO")
                .owners_only(true))
        )
    );

    let shard_manager = client.shard_manager.clone();

    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));

            let lock = shard_manager.lock();
            let shard_runners = lock.runners.lock();

            for (id, runner) in shard_runners.iter() {
                println!(
                    "Shard ID {} is {} with a latency of {:?}",
                    id,
                    runner.stage,
                    runner.latency,
                );
            }
        }
    });

    if let Err(why) = client.start_shards(4) {
        error!("Client error: {:?}", why);
    }
}

fn get_command_prefix(ctx: &Context) -> String {
    let data = ctx.data.lock();
    let settings = data.get::<Settings>().unwrap().lock().unwrap();
    settings.command_prefix.clone()
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

//fn is_running_as_docker_container() -> bool {
//    !std::env::var("DOCKER_ENV").is_err()
//}
