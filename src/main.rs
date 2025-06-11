mod command;
mod database;

#[macro_use]
extern crate sqlx;

use std::collections::HashMap;
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
	regex_captures: usize,
	db_handler: DatabaseHandler,
}

macro_rules! add_commands {
	($($cmd:ident => $exec:ident), * $(,)?) => {
		impl Handler {
			async fn register_commands(&self, ctx: &Context) -> Result<(), SerenityError> {
				println!("Registering commands...");

				let old_commands = Command::get_global_commands(&ctx).await.unwrap_or_default();
				let mut old_commands: HashMap<_, _> = old_commands
					.into_iter()
					.map(|cmd| (cmd.name.to_owned(), cmd))
					.collect();
				println!("Got {} old commands", old_commands.len());

				let mut handles = Vec::new();
				$(
					match old_commands
						.remove($cmd::NAME)
						.map(|old_cmd| ($cmd == old_cmd, old_cmd))
					{
						None => {
							println!("Command {} does not exist, creating..", $cmd);
							handles.push(tokio::spawn(Command::create_global_command(
								ctx.to_owned(),
								$cmd::into_command(),
							)));
						}
						Some((false, old_cmd)) => {
							println!("Command {} was modified, editing..", $cmd);
							handles.push(tokio::spawn(Command::edit_global_command(
								ctx.to_owned(),
								old_cmd.id,
								$cmd::into_command(),
							)));
						}
						Some((true, _)) => {
							println!(
								"Command {} was not modified, keeping the same",
								$cmd
							);
						}
					}
				)*

				for handle in handles {
					match handle.await {
						Err(why) => panic!("Future could not complete\n{why}"),
						Ok(Err(why)) => return Err(why),
						Ok(Ok(cmd)) => println!("Registered command {}", cmd.name),
					};
				}

				let unused_commands: Vec<_> = old_commands.into_values().collect();
				let mut handles = Vec::new();
				for unused_command in unused_commands {
					let ctx = ctx.to_owned();
					handles.push(
						tokio::spawn(async move {
							Command::delete_global_command(ctx, unused_command.id).await?;
							Ok(unused_command.name)
						})
					);
				}
				for handle in handles {
					match handle.await {
						Err(why) => panic!("Future could not complete\n{why}"),
						Ok(Err(why)) => return Err(why),
						Ok(Ok(cmd)) => println!("Deleted unused command {cmd}"),
					};
				}

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
		if msg.author.bot {
			return;
		}
		let Some(server_id) = msg.guild_id else {
			return;
		};
		let server_id = server_id.get();
		let author_id = msg.author.id.get();

		if let Some(found) = self.regex.captures(&msg.content.to_lowercase()) {
			let Some(emote) = (1..=self.regex_captures).flat_map(|i| found.get(i)).next() else {
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
					format!("You have ended your message with *{emote}* **{new_count}** times!",),
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
			eprintln!("Cannot respond to slash command: {why}");
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

	let regex = include_str!("regex.txt").trim();
	let regex_captures = regex.chars().filter(|c| *c == '(').count();
	let regex = Regex::new(regex).expect("Expected a valid regex expression");

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let handler = Handler {
		regex,
		regex_captures,
		db_handler,
	};

	let mut client = Client::builder(&token, intents)
		.event_handler(handler)
		.await
		.expect("Err creating client");

	if let Err(why) = client.start().await {
		eprintln!("Client error: {why:?}");
	}
}
