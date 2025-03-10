mod command;
mod database;

#[macro_use]
extern crate sqlx;

use std::env;

use command::{all::*, IntoCommand};
use database::{DatabaseHandler, VerboseLevel};
use dotenvy::dotenv;
use regex::Regex;
use serenity::all::{Command, CommandInteraction, CreateInteractionResponse, Interaction};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use sqlx::Pool;

struct Handler {
	regex: Regex,
	db_handler: DatabaseHandler,
}

macro_rules! add_commands {
	($($cmd:ident => $exec:ident), * $(,)?) => {
		impl Handler {
			async fn register_commands(&self, ctx: &Context) -> Result<(), SerenityError> {
				tokio::try_join!(
					$(
						Command::create_global_command(&ctx, $cmd::into_command())
					), *
				)?;

				println!("Registered commands:");
				$(
					println!("{}", $cmd);
				)*

				Ok(())
			}

			async fn run_command(
				&self,
				cmd: &CommandInteraction,
			) -> sqlx::Result<Option<CreateInteractionResponse>> {
				let msg = match cmd.data.name.as_str() {
					$(
						$cmd::NAME => $exec(&self.db_handler, cmd).await?,
					)*
					_ => return Ok(None),
				};

				Ok(Some(CreateInteractionResponse::Message(msg)))
			}
		}
	};
}

add_commands!(
	OptInCommand => opt_in,
	OptOutCommand => opt_out,
	SilentCommand => silent,
	VerboseCommand => verbose,
	CountsCommand => counts,
	LeaderboardCommand => leaderboard,
	MuteAllCommand => mute_all,
	UnmuteAllCommand => unmute_all,
);

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, ctx: Context, ready: Ready) {
		if let Err(why) = self.register_commands(&ctx).await {
			panic!("Could not register commands!\n{why}");
		}

		println!("{} is connected!", ready.user.name);
	}

	async fn message(&self, ctx: Context, msg: Message) {
		let Some(server_id) = msg.guild_id else {
			return;
		};
		let server_id = server_id.get();
		let author_id = msg.author.id.get();

		if let Some(found) = self.regex.captures(&msg.content) {
			let Some(emote) = found.get(1) else {
				return;
			};
			let emote = emote.as_str();

			let new_count = match self.db_handler.add_one(author_id, server_id, emote).await {
				Ok(Some(new_count)) => new_count,
				Ok(None) => return,
				Err(why) => return eprintln!("DB error: {why}"),
			};

			match self.db_handler.verbose_level(author_id, server_id).await {
				Ok(VerboseLevel::Verbose) => (),
				Ok(VerboseLevel::Silent) => return,
				Ok(VerboseLevel::Every(every)) => {
					if new_count % every.get() != 0 {
						return;
					}
				}
				Err(why) => return eprintln!("DB error: {why}"),
			};

			let try_reply = msg
				.reply(
					&ctx,
					format!("You have ended your message with '{emote}' **{new_count}** times!",),
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

		let response = match self.run_command(&command).await {
			Ok(Some(r)) => r,
			Ok(None) => return,
			Err(why) => return eprintln!("DB error: {why}"),
		};

		if let Err(why) = command.create_response(&ctx.http, response).await {
			println!("Cannot respond to slash command: {why}");
		}
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	let token = env::var("BOT_TOKEN").expect("Expected a BOT_TOKEN in the environment");
	let db_url = env::var("DATABASE_URL").expect("Expected a DATABASE_URL in the environment");

	let pool = Pool::connect(&db_url).await.unwrap();
	let db_handler = DatabaseHandler::new(pool);

	let regex = Regex::new(r#"([:;xX]3)3*c*$"#).unwrap();

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let handler = Handler { regex, db_handler };

	let mut client = Client::builder(&token, intents)
		.event_handler(handler)
		.await
		.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {why:?}");
	}
}
