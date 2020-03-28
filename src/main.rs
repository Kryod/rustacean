#[macro_use] extern crate log;
#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;

pub mod commands;
pub mod lang_manager;
pub mod tools;
pub mod schema;
pub mod models;
pub mod dbl;
pub mod file_logger;
mod test;

use commands::*;
use lang_manager::LangManager;

use serenity::{
    http::{ self, client::Http },
    client::bridge::gateway::ShardManager,
    prelude::{ Client, Context, EventHandler },
    model::{
        channel::Embed,
        prelude::{ Ready, Message, ResumedEvent, UserId },
    },
    framework::standard::{
        DispatchError, StandardFramework, Args, CommandGroup, HelpOptions, CommandResult, CommandOptions, CheckResult, help_commands,
        macros::{ group, check, help },
    },
};
use diesel::{
    SqliteConnection,
    r2d2::{ ConnectionManager, Pool },
};
use typemap::Key;

use std::{
    sync::{ Arc, Mutex },
    io::{ Error, ErrorKind, Read },
    collections::{ HashSet, HashMap },
    iter::FromIterator, str::FromStr, process::Command,
};

// A container type is created for inserting into the Client's `data`, which
// allows for data to be accessible across all events and framework commands, or
// anywhere else that has a copy of the `data` Arc.
struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<serenity::prelude::Mutex<ShardManager>>;
}

#[derive(Default, Deserialize, Clone)]
pub struct Settings {
    pub discord_token: String,
    pub dbl_api_key: Option<String>,
    pub command_prefix: String,
    pub log_level_term: String,
    pub log_level_file: String,
    pub log_file: String,
    pub db_connection_pool_size: u32,
    pub bot_owners: Vec<serenity::model::prelude::UserId>,
    pub webhook_id: Option<u64>,
    pub webhook_token: Option<String>,
    pub webhook_frequency: Option<u64>,
    pub webhook_role: Option<String>,
    pub cpu_load: String,
    pub ram_load: String,
    pub kernel_memory: String,
    pub compilation_timeout: u64,
    pub execution_timeout: u64,
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
                },
                1 => presence_status_thread(ready.user.id, ctx),
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
        let ctx = ctx.lock().unwrap();
        let data = ctx.data.read();
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

            let guilds = get_guilds(&ctx.lock().unwrap());
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

fn cargo_test_thread(settings: Settings) {
    let (webhook_id, webhook_token, webhook_freq, webhook_role, discord_token) = {
        (
            settings.webhook_id,
            settings.webhook_token,
            settings.webhook_frequency,
            settings.webhook_role,
            settings.discord_token,
        )
    };
    if webhook_id.is_none() || webhook_token.is_none() || webhook_freq.is_none() || webhook_role.is_none() {
        return;
    }

    std::thread::spawn(move || {
        // Periodic tests to check if bot is broken
        let (webhook_id, webhook_token, webhook_freq, webhook_role) = (
            webhook_id.unwrap(),
            webhook_token.unwrap(),
            webhook_freq.unwrap(),
            webhook_role.unwrap(),
        );

        let test_freq = std::time::Duration::from_secs(60 * webhook_freq);
        let http_client = Http::new_with_token(&discord_token);

        loop {
            info!("Running test command!");

            let webhook = http_client.get_webhook_with_token(webhook_id, &webhook_token).expect("Invalid webhook");

            let mut cargo = Command::new("cargo");
            let cargo_test = if cfg!(debug_assertions) {
                cargo.arg("test")
            } else {
                cargo.arg("test").arg("--release")
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
                    let _ = webhook.execute(&http_client, false, |w| w
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

            let content = if ping {
                format!("<@&{}>, we have a problem!", webhook_role)
            } else {
                "Everything is fine :sunny:".into()
            };

            let _ = webhook.execute(&http_client, false, |w| w
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
            if let Ok(snippets_dir) = commands::exec::get_snippets_directory() {
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
            }

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

fn get_guilds(ctx: &Context) -> Result<usize, serenity::Error> {
    let mut count = 0;
    let mut last_guild_id = 0;
    loop {
        let guilds = ctx.http.get_guilds(&http::GuildPagination::After(last_guild_id.into()), 100)?;
        let len = guilds.len();
        count += len;
        if len < 100 {
            break;
        }
        if let Some(last) = guilds.last() {
            last_guild_id = *last.id.as_u64();
        }
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
    use simplelog::{ CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode };

    let config = ConfigBuilder::new()
                            .set_time_format_str("[%Y-%m-%d %H:%M:%S]")
                            .build();

    let log_level_term = LevelFilter::from_str(settings.log_level_term.as_ref()).expect("Invalid log level filter");
    let log_level_file = LevelFilter::from_str(settings.log_level_file.as_ref()).expect("Invalid log level filter");

    CombinedLogger::init(
        vec![
            TermLogger::new(log_level_term, config, TerminalMode::Mixed).unwrap(),
            Box::new(file_logger::FileLogger::new(&settings.log_file, log_level_file)),
        ]
    ).unwrap();
}

#[help]
fn rustacean_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

#[check]
#[name = "Admin"]
// Whether the check shall be tested in the help-system.
#[check_in_help(true)]
// Whether the check shall be displayed in the help-system.
#[display_in_help(true)]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }

    false.into()
}

#[group]
#[commands(git, support, invite, exec, languages, versions, stats)]
#[description = ":desktop: Basic"]
struct General;

#[group]
#[checks(Admin)]
#[commands(ban, unban)]
#[description = ":star: Administrator"]
struct Admin;

#[group]
#[owners_only]
#[commands(logs)]
#[description = ":robot: Bot owner"]
struct Owner;

fn main() {
    if tools::tools() {
        return;
    }

    if !std::path::PathBuf::from("rustacean.sqlite3").exists() {
        tools::update_db::update_db();
    }

    let settings = init_settings();
    let command_prefix = settings.command_prefix.clone();
    let owners = HashSet::from_iter(settings.bot_owners.clone());
    init_logging(&settings);

    let mut client = Client::new(&settings.discord_token, Handler).expect("Err creating client");

    let manager: ConnectionManager<SqliteConnection> = ConnectionManager::new("rustacean.sqlite3");
    let pool = Pool::builder()
        .max_size(settings.db_connection_pool_size)
        .build(manager)
        .expect("Could not build database connection pool.");
    let pool = Arc::new(pool);

    models::Ban::cleanup_outdated_bans(&pool);

    let mut lang_manager = LangManager::new();
    lang_manager.check_available_languages();

    {
        let mut data = client.data.write();
        data.insert::<Settings>(Arc::new(Mutex::new(settings)));
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<LangManager>(Arc::new(Mutex::new(lang_manager)));
        data.insert::<DbPool>(pool.clone());
        data.insert::<Bans>(models::Ban::get_bans(&pool));
    }

    client.with_framework(StandardFramework::new()
    .configure(|c| c
            .prefix(&command_prefix)
            .owners(owners))
        .before(|ctx, msg, _cmd_name| {
            let data = ctx.data.read();
            let bans = data.get::<Bans>().unwrap();
            match bans.get(&msg.author.id) {
                Some(bans) => {
                    let banned = bans.iter().any(|ban| {
                        ban.is_banned_for_guild(msg.guild_id)
                    });
                    if banned {
                        let _ = msg.reply(&ctx, "You cannot run commands while being banned.");
                    }
                    !banned
                },
                None => true,
            }
        })
        // Set a function that's called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|ctx, msg, error| {
            match error {
                DispatchError::Ratelimited(seconds) => {
                    let _ = msg.reply(ctx, &format!("Try this again in {} seconds.", seconds));
                },
                DispatchError::OnlyForOwners | DispatchError::LackingRole | DispatchError::BlockedUser | DispatchError::LackingPermissions(_) => {
                    let _ = msg.reply(ctx, "You are not allowed to do this.");
                },
                DispatchError::BlockedGuild => {
                    let _ = msg.reply(ctx, "Rustacean is not available on this server because its owner has been banned.");
                },
                _ => {},
            };
        })
        .help(&RUSTACEAN_HELP)
        // Time out for exec: Can't be used more than 2 times per 30 seconds, with a 5 second delay
        //.bucket("exec_bucket", 5, 30, 2)
        // Can't be used more than once per 5 seconds:
        .bucket("exec_bucket", |b| b.delay(5))
        .group(&GENERAL_GROUP)
        .group(&ADMIN_GROUP)
        .group(&OWNER_GROUP)
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

    snippets_cleanup_thread();
    cargo_test_thread(init_settings());

    if let Err(why) = client.start_shards(2) {
        error!("Client error: {:?}", why);
    }
}

fn get_command_prefix(ctx: &Context) -> String {
    let data = ctx.data.read();
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
    let game = serenity::model::gateway::Activity::playing(game_name);
    let status = serenity::model::user::OnlineStatus::Online;
    ctx.set_presence(Some(game), status);
}
