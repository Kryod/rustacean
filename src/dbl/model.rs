#[derive(Debug, Default, Clone)]
pub struct Social {
    /// The youtube channel id of the user
    pub youtube: Option<String>,
    /// The reddit username of the user
    pub reddit: Option<String>,
    /// The twitter username of the user
    pub twitter: Option<String>,
    /// The instagram username of the user
    pub instagram: Option<String>,
    /// The github username of the user
    pub github: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct User {
    /// The id of the user
    pub id: String,
    /// The username of the user
    pub username: String,
    /// The discriminator of the user
    pub discriminator: String,
    /// The avatar hash of the user's avatar
    pub avatar: Option<String>,
    /// The cdn hash of the user's avatar if the user has none
    pub def_avatar: String,
    /// The bio of the user
    pub bio: Option<String>,
    /// The banner image url of the user
    pub banner: Option<String>,
    /// The social usernames of the user
    pub social: Social,
    /// The custom hex color of the user
    pub color: Option<String>,
    /// The supporter status of the user
    pub is_supporter: bool,
    /// The certified status of the user
    pub is_certified_dev: bool,
    /// The mod status of the user
    pub is_mod: bool,
    /// The website moderator status of the user
    pub is_web_mod: bool,
    /// The admin status of the user
    pub is_admin: bool,
}

#[derive(Debug, Clone)]
pub struct Bot {
    /// The id of the bot
    pub id: String,
    /// The username of the bot
    pub username: String,
    /// The discriminator of the bot
    pub discriminator: String,
    /// The avatar hash of the bot's avatar
    pub avatar: Option<String>,
    /// The cdn hash of the bot's avatar if the bot has none
    pub def_avatar: String,
    /// The library of the bot
    pub lib: String,
    /// The prefix of the bot
    pub prefix: String,
    /// The short description of the bot
    pub shortdesc: String,
    /// The long description of the bot. Can contain HTML and/or Markdown
    pub longdesc: Option<String>,
    /// The tags of the bot
    pub tags: Vec<String>,
    /// The website url of the bot
    pub website: Option<String>,
    /// The support server invite code of the bot
    pub support: Option<String>,
    /// The link to the github repo of the bot
    pub github: Option<String>,
    /// The owners of the bot. First one in the array is the main owner
    pub owners: Vec<String>,
    /// The custom bot invite url of the bot
    pub invite: Option<String>,
    /// The date when the bot was approved
    pub date: chrono::naive::NaiveDate,
    /// The certified status of the bot
    pub is_certified_bot: bool,
    /// The vanity url of the bot
    pub vanity: Option<String>,
    /// The amount of upvotes the bot has
    pub points: usize,
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    /// The amount of servers the bot is in
    pub server_count: Option<usize>,
    /// The amount of servers the bot is in per shard
    pub shards: Vec<usize>,
    /// The amount of shards the bot has
    pub shard_count: Option<usize>,
}
