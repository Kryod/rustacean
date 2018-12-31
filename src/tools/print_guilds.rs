use std::thread;
use std::time::Duration;
use std::sync::{ Arc, Mutex };

use typemap::Key;
use serenity::model::prelude::{ Ready, Guild };
use serenity::prelude::{ Client, EventHandler, Context };

struct GuildReadyCounter {
    init: bool,
    total_guilds: usize,
    ready_guilds: usize,
}

impl Key for GuildReadyCounter {
    type Value = Arc<Mutex<GuildReadyCounter>>;
}

impl GuildReadyCounter {
    pub fn new() -> Self {
        Self {
            init: false,
            total_guilds: 0,
            ready_guilds: 0,
        }
    }

    pub fn set_total(&mut self, total: usize) {
        self.init = true;
        self.total_guilds = total;
    }

    pub fn add_ready(&mut self) {
        self.ready_guilds += 1;
    }

    pub fn all_ready(&self) -> bool {
        self.init && self.ready_guilds >= self.total_guilds
    }
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        let guilds = ready.guilds.len();
        let mut data = ctx.data.lock();
        let mut counter = data.get_mut::<GuildReadyCounter>().unwrap().lock().unwrap();
        counter.set_total(guilds);
    }

    fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if !is_new {
            let mut data = ctx.data.lock();
            let mut counter = data.get_mut::<GuildReadyCounter>().unwrap().lock().unwrap();
            counter.add_ready();
        }
        println!("  - {} (id: {}, {} users)", guild.name, guild.id, guild.members.len());
    }
}

pub fn print_guilds() {
    let counter = Arc::new(Mutex::new(GuildReadyCounter::new()));

    let thread_counter = counter.clone();
    thread::spawn(move || {
        let settings = ::init_settings();
        let mut client = Client::new(&settings.discord_token, Handler).expect("Err creating client");
        {
            let mut data = client.data.lock();
            data.insert::<GuildReadyCounter>(thread_counter);
        }
        client.start().expect("Client error");
    });

    loop {
        let done = {
            counter.lock().unwrap().all_ready()
        };
        if done {
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
}
