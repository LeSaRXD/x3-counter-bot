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

const OPT_IN: &str = "opt_in";
const OPT_OUT: &str = "opt_out";
const SILENT: &str = "silent";
const VERBOSE: &str = "verbose";

impl Handler {
	async fn register_commands(&self, ctx: &Context) -> Result<(), SerenityError> {
		let opt_in = CreateCommand::new(OPT_IN).description("Start tracking 'x3's");
		Command::create_global_command(&ctx, opt_in).await?;
		println!("Registered {OPT_IN} command");

		let opt_out = CreateCommand::new(OPT_OUT).description("Stop tracking 'x3's");
		Command::create_global_command(&ctx, opt_out).await?;
		println!("Registered {OPT_OUT} command");

		let silent =
			CreateCommand::new(SILENT).description("Track 'x3's silently (don't send messages)");
		Command::create_global_command(&ctx, silent).await?;
		println!("Registered {SILENT} command");

		let verbose =
			CreateCommand::new(VERBOSE).description("Track 'x3's verbosely (do send messages)");
		Command::create_global_command(&ctx, verbose).await?;
		println!("Registered {VERBOSE} command");

		Ok(())
	}
}

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, ready: Ready) {
		// commands
		if let Err(why) = self.register_commands(&ctx).await {
			panic!("Could not register commands!\n{why}");
		}

		println!("{} is connected!", ready.user.name);
	}

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

			match self.db_handler.is_silent(msg.author.id.get()).await {
				Ok(true) => return,
				Err(why) => return eprintln!("DB error: {why}"),
				_ => (),
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
		let Interaction::Command(command) = interaction else {
			return;
		};
		let user_id = command.user.id.get();

		let content = match command.data.name.as_str() {
			OPT_IN => {
				if let Err(e) = self.db_handler.set_opt_out(user_id, false).await {
					return eprintln!("DB error: {e}");
				};
				"I will count your ':3's now uwu"
			}
			OPT_OUT => {
				if let Err(e) = self.db_handler.set_opt_out(user_id, true).await {
					return eprintln!("DB error: {e}");
				};
				"I won't count your ':3's now qwq"
			}
			SILENT => {
				if let Err(e) = self.db_handler.set_silent(user_id, true).await {
					return eprintln!("DB error: {e}");
				};
				"I won't respond to your messages but will still count 'x3's"
			}
			VERBOSE => {
				if let Err(e) = self.db_handler.set_silent(user_id, false).await {
					return eprintln!("DB error: {e}");
				};
				"I will now respond to your messages"
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
