#![feature(async_closure)]

use convert_case::{Case, Casing};
use dotenvy::dotenv;
use serenity::Client;
use serenity::all::{ChannelId, GatewayIntents, GuildId};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;

#[derive(serde::Serialize)]
struct Message {
    content: String,
}

async fn handle_client(stream: UnixStream, client: Arc<Client>, channel_id: ChannelId) {
    // let channel_id = client.http.get_guilds(None, Some(100));

    println!("reading stream");
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        let text = line.unwrap();
        println!("{}", &text);
        // let response = channel_id.say(client.http.http(), text).await;
        let response = client
            .http
            .send_message(channel_id, vec![], &Message { content: text })
            .await;
        match response {
            Ok(r) => println!("sent message {:?}", r.content),
            Err(e) => println!("error sending message {:?}", e),
        }
        // client.http.send_message(channel_id, files, map)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    _ = dotenv();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS;
    let guild_id = GuildId::new(
        std::env::var("GUILD_ID")
            .expect("missing GUILD_ID")
            .parse()
            .expect("couldn't parse GUILD_ID"),
    );

    let client = serenity::Client::builder(token.clone(), intents)
        .await
        .expect("error creating client");
    // let guild_response = client.http.get_guild(guild_id).await.unwrap();
    // println!("guild {:?}", guild_response);
    let channels_response = client.http.get_channels(guild_id).await;
    let mut channel_map = HashMap::new();
    if let Ok(channels) = channels_response {
        // println!("got channels {channels:?}");
        for g in channels {
            channel_map.insert(g.name.to_case(Case::Snake), g.id);
        }
    } else {
        panic!("err getting channels {channels_response:?}")
    }

    let channel = "curium".to_case(Case::Snake);
    let socket = "/tmp/discord/".to_owned() + channel.as_str();
    if fs::exists(&socket).is_ok_and(|f| f) {
        let _ = fs::remove_file(&socket);
    }
    let listener = UnixListener::bind(&socket)?;
    let channel_id = channel_map.get(&channel).unwrap().to_owned();

    let boxed_client = Arc::new(client);

    println!("listening on {:?}", &socket);
    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                let client = boxed_client.clone();
                println!("spawning handler with channel id {:?}", &channel_id);
                tokio::spawn(async move { handle_client(stream, client, channel_id).await });
            }
            Err(err) => {
                println!("Error {err:?}");
                /* connection failed */
                break;
            }
        }
    }
    Ok(())
}
