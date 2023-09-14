use clap::Parser;
use discord_openai::discord;
use futures::future::join_all;
use std::{error::Error, panic::resume_unwind, sync::Arc};
use tracing::Level;
use dotenv::dotenv;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Default, Debug)]
pub struct Args {
    #[clap(short, long, help = "OPEN_API_KEY")]
    pub openai_api_key: Option<String>,
    #[clap(short, long, help = "BOT_TOKEN")]
    pub bot_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    dotenv().expect(".env file not found");

    let args = Args::parse();

    let (openai_api_key, bot_token) = if args.openai_api_key.is_some() && args.bot_token.is_some() {
        (args.openai_api_key.unwrap(), args.bot_token.unwrap())
    } else {
        get_discord_openai_environment_variables()
    };

    //Discord-openai is still under development. This Vec<JoinHandle> is here to allow for other potential services to be easily spun up in separate threads
    let mut handles = vec![];

    let openai_client = Arc::new(openairs::client::OpenAIClient::new(openai_api_key));
    handles.push(
        discord::DiscordService::new(bot_token, openai_client)
            .spawn()
            .await,
    );

    //Handle any errors from in progress tasks
    for res in join_all(handles).await {
        match res {
            Ok(discord_gpt_result) => match discord_gpt_result {
                Ok(_) => {}
                Err(err) => {
                    panic!("{err:?}");
                }
            },
            Err(err) => {
                {
                    if err.is_panic() {
                        // Resume the panic on the main task
                        resume_unwind(err.into_panic());
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn init_tracing() {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

fn get_discord_openai_environment_variables() -> (String, String) {

    if let Ok(open_api_key) = std::env::var("OPEN_API_KEY") {
        if let Ok(bot_token) = std::env::var("BOT_TOKEN") {
            (open_api_key, bot_token)
        } else {
            panic!("Discord bot token not found in command line arguments or environment variables")
        }
    } else {
        panic!("Openai API key not found in command line arguments or environment variables")
    }
}

// MTE1MTAxOTM0ODQ2NzY1NDcwNg.GOVYdL.MJynyn7Jl8Z38lr-bbggxkf0zF99jwEoRCd6s4
// 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