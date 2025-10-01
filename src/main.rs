use dotenvy::dotenv;
use serenity::all::{GatewayIntents, GuildId};
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::{fs, thread};

async fn handle_client(stream: UnixStream) {
    // let channel_id = client.http.get_guilds(None, Some(100));

    let stream = BufReader::new(stream);
    for line in stream.lines() {
        println!("{}", line.unwrap());
        // client.http.send_message(channel_id, files, map)
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    _ = dotenv();

    let socket = "/tmp/discord/general";
    if fs::exists(socket).is_ok_and(|f| f) {
        let _ = fs::remove_file(socket);
    }
    let listener = UnixListener::bind(socket)?;

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let guild_id = GuildId::new(
        std::env::var("GUILD_ID")
            .expect("missing GUILD_ID")
            .parse()
            .expect("couldn't parse GUILD_ID"),
    );

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("building client");
                let mut client = serenity::Client::builder(&token.clone(), intents)
                    .await
                    .expect("error creating client");
                println!("starting client");
                // if let Err(oops) = client.start().await {
                //     panic!("Failed to start client {oops:?}")
                // }
                println!("getting guilds");
                let channels_response = client.http.get_channels(guild_id).await;
                if let Ok(channels) = channels_response {
                    println!("got channels {channels:?}");
                    for g in channels {
                        println!("{g:?}")
                    }
                } else {
                    println!("err getting channels {channels_response:?}")
                }

                /* connection succeeded */
                thread::spawn(|| handle_client(stream));
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
