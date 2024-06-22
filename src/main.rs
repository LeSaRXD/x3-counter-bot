mod database;

#[macro_use]
extern crate sqlx;

use std::env;

use database::DatabaseHandler;
use dotenv::dotenv;
use regex::Regex;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use sqlx::Pool;

struct Handler {
	general_regex: Regex,
	specific_regex: Regex,
	db_handler: DatabaseHandler,
}

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		if let Some(found) = self.general_regex.find(&msg.content) {
			let emote = match self.specific_regex.find(found.as_str()) {
				Some(res) => res.as_str(),
				None => {
					eprintln!("Something is wrong with the regex!");
					return;
				}
			};

			let new_count = match self.db_handler.add_one(msg.author.id.into(), emote).await {
				Ok(new_count) => new_count,
				Err(e) => {
					eprintln!("DB error: {e}");
					return;
				}
			};

			let try_reply = msg
				.reply(
					&ctx,
					format!("You have ended your message with `{emote}` **{new_count}** times!"),
				)
				.await;
			if let Err(why) = try_reply {
				eprintln!("Cound not send message: {why}");
			}
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	let token = env::var("BOT_TOKEN").expect("Expected a BOT_TOKEN in the environment");
	let db_url = env::var("DATABASE_URL").expect("Expected a DATABASE_URL in the environment");
	let pool = Pool::connect(&db_url).await.unwrap();
	let db_handler = DatabaseHandler::new(pool);

	let general = Regex::new(r#"[:;xX]3+c?$"#).unwrap();
	let specific = Regex::new(r#"[:;xX]3"#).unwrap();

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let handler = Handler {
		general_regex: general,
		specific_regex: specific,
		db_handler,
	};

	let mut client = Client::builder(&token, intents)
		.event_handler(handler)
		.await
		.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {why:?}");
	}
}
