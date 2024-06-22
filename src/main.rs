mod database;

#[macro_use]
extern crate sqlx;

use std::env;

use database::DatabaseHandler;
use dotenv::dotenv;
use regex::Regex;
use serenity::all::{
	Command, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
	Interaction,
};
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
				None => return eprintln!("Something is wrong with the regex!"),
			};

			let new_count = match self.db_handler.add_one(msg.author.id.get(), emote).await {
				Ok(Some(new_count)) => new_count,
				Ok(None) => return,
				Err(why) => return eprintln!("DB error: {why}"),
			};

			let try_reply = msg
				.reply(
					&ctx,
					format!("You have ended your message with '{emote}' **{new_count}** times!"),
				)
				.await;
			if let Err(why) = try_reply {
				eprintln!("Cound not send message: {why}");
			}
		}
	}

	async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
		if let Interaction::Command(command) = interaction {
			let content = match command.data.name.as_str() {
				"opt_in" => {
					if let Err(e) = self
						.db_handler
						.set_opt_out(command.user.id.get(), false)
						.await
					{
						return eprintln!("DB error: {e}");
					};
					"I will count your ':3's now uwu"
				}
				"opt_out" => {
					if let Err(e) = self
						.db_handler
						.set_opt_out(command.user.id.get(), true)
						.await
					{
						return eprintln!("DB error: {e}");
					};
					"I won't count your ':3's now qwq"
				}
				_ => return,
			};

			let data = CreateInteractionResponseMessage::new().content(content);
			let builder = CreateInteractionResponse::Message(data);
			if let Err(why) = command.create_response(&ctx.http, builder).await {
				println!("Cannot respond to slash command: {why}");
			}
		}
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		// commands
		let opt_in = CreateCommand::new("opt_in").description("Start tracking 'x3's");
		if let Err(why) = Command::create_global_command(&ctx, opt_in).await {
			return eprintln!("Error creating opt_in command: {why}");
		} else {
			println!("Registered opt_in command");
		}
		let opt_out = CreateCommand::new("opt_out").description("Stop tracking 'x3's");
		if let Err(why) = Command::create_global_command(&ctx, opt_out).await {
			return eprintln!("Error creating opt_out command: {why}");
		} else {
			println!("Registered opt_out command");
		}

		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	// env
	dotenv().ok();
	let token = env::var("BOT_TOKEN").expect("Expected a BOT_TOKEN in the environment");
	let db_url = env::var("DATABASE_URL").expect("Expected a DATABASE_URL in the environment");

	// db
	let pool = Pool::connect(&db_url).await.unwrap();
	let db_handler = DatabaseHandler::new(pool);

	// regex
	let general = Regex::new(r#"[:;xX]3+c*$"#).unwrap();
	let specific = Regex::new(r#"[:;xX]3"#).unwrap();

	// intents
	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	// event handler
	let handler = Handler {
		general_regex: general,
		specific_regex: specific,
		db_handler,
	};

	// client
	let mut client = Client::builder(&token, intents)
		.event_handler(handler)
		.await
		.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {why:?}");
	}
}
