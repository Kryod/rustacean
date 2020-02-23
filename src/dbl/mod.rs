extern crate reqwest;

pub mod model;
//use self::model::*;

use std::collections::HashMap;

use serenity::model::id::UserId;

type RequestType<T> = Result<T, reqwest::Error>;

pub fn get_user(_user_id: &str) /*-> RequestType<User>*/ {
    // TODO...
    // /users/{user_id}
}

pub fn get_bots() /*-> RequestType<Vec<Bot>>*/ {
    // TODO...
    // /bots
}

pub fn get_bot(_bot_id: UserId) /*-> RequestType<Bot>*/ {
    // TODO...
    // /bots/{bot_id}
}

pub fn vote_check(_bot_id: UserId) /*-> RequestType<bool>*/ {
    // TODO...
    // /bots/{bot_id?}/check
}

pub fn get_stats(_bot_id: UserId) /*-> RequestType<Stats>*/ {
    // TODO...
    // /bots/{bot_id}/stats
}

pub fn post_stats(bot_id: UserId, api_key: &str, server_count: usize) -> RequestType<String> {
    let mut data = HashMap::new();
    data.insert("server_count", server_count.to_string());
    do_request(&format!("/bots/{}/stats", bot_id), api_key, Some(data))
}

pub fn post_stats_shards(bot_id: UserId, api_key: &str, server_count: Vec<usize>) -> RequestType<String> {
    let mut data = HashMap::new();
    data.insert("server_count", server_count.iter().map(|count| count.to_string()).collect::<Vec<String>>().join(", "));
    data.insert("shard_count", server_count.len().to_string());
    do_request(&format!("/bots/{}/stats", bot_id), api_key, Some(data))
}

fn do_request(endpoint: &str, api_key: &str, data: Option<HashMap<&str, String>>) -> RequestType<String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://discordbots.org/api{}", endpoint);
    let builder = match data {
        Some(data) => {
            client.post(&url)
                .json(&data)
        },
        None => client.get(&url),
    };
    let res = builder
        .header(reqwest::header::AUTHORIZATION, api_key)
        .send()?
        .text()?;
    Ok(res)
}
