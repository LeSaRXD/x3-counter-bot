mod database;

#[macro_use]
extern crate sqlx;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env;

use database::{DatabaseHandler, LeaderboardRow};
use dotenvy::dotenv;
use regex::Regex;
use serenity::all::{
	Command, CommandDataOptionValue, CommandInteraction, CommandOptionType, CreateAllowedMentions,
	CreateCommand, CreateCommandOption, CreateInteractionResponse,
	CreateInteractionResponseMessage, Interaction, InteractionResponseFlags, Permissions,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use sqlx::Pool;

struct Handler {
	regex: Regex,
	db_handler: DatabaseHandler,
}

const OPT_IN: &str = "opt_in";
const OPT_OUT: &str = "opt_out";
const SILENT: &str = "silent";
const VERBOSE: &str = "verbose";
const COUNTS: &str = "counts";
const LEADERBOARD: &str = "leaderboard";
const MUTE_ALL: &str = "mute_all";
const UNMUTE_ALL: &str = "unmute_all";

impl Handler {
	async fn register_commands(&self, ctx: &Context) -> Result<(), SerenityError> {
		let opt_in = CreateCommand::new(OPT_IN).description("Start tracking 'x3's");

		let opt_out = CreateCommand::new(OPT_OUT).description("Stop tracking 'x3's");

		let silent =
			CreateCommand::new(SILENT).description("Track 'x3's silently (don't send messages)");

		let verbose =
			CreateCommand::new(VERBOSE).description("Track 'x3's verbosely (do send messages)");

		let counts = CreateCommand::new(COUNTS).description("Get your 'x3' counts");

		let count_arg = CreateCommandOption::new(
			CommandOptionType::Integer,
			"count",
			"Max count of users per emote",
		)
		.required(false)
		.min_int_value(1)
		.max_int_value(5);
		let leaderboard = CreateCommand::new(LEADERBOARD)
			.description("Get the 'x3' leaderboard")
			.add_option(count_arg)
			.dm_permission(false);

		let mute_all = CreateCommand::new(MUTE_ALL)
			.description("Mute all count messages in the server")
			.default_member_permissions(Permissions::MANAGE_MESSAGES)
			.dm_permission(false);

		let unmute_all = CreateCommand::new(UNMUTE_ALL)
			.description("Unmute all count messages in the server")
			.default_member_permissions(Permissions::MANAGE_MESSAGES)
			.dm_permission(false);

		tokio::try_join!(
			Command::create_global_command(&ctx, opt_in),
			Command::create_global_command(&ctx, opt_out),
			Command::create_global_command(&ctx, silent),
			Command::create_global_command(&ctx, verbose),
			Command::create_global_command(&ctx, counts),
			Command::create_global_command(&ctx, leaderboard),
			Command::create_global_command(&ctx, mute_all),
			Command::create_global_command(&ctx, unmute_all),
		)?;
		println!(
			r#"Registered commands:
{OPT_IN}
{OPT_OUT}
{SILENT}
{VERBOSE}
{COUNTS}
{LEADERBOARD}
{MUTE_ALL}
{UNMUTE_ALL}"#
		);
		Ok(())
	}

	async fn run_command(
		&self,
		cmd: &CommandInteraction,
	) -> sqlx::Result<Option<CreateInteractionResponse>> {
		let user_id = cmd.user.id.get();
		let msg = match cmd.data.name.as_str() {
			OPT_IN => {
				self.db_handler.set_opt_out(user_id, false).await?;
				CreateInteractionResponseMessage::new()
					.content("I will count your ':3's now UwU")
					.flags(InteractionResponseFlags::EPHEMERAL)
			}
			OPT_OUT => {
				self.db_handler.set_opt_out(user_id, true).await?;
				CreateInteractionResponseMessage::new()
					.content("I won't count your ':3's now qwq")
					.flags(InteractionResponseFlags::EPHEMERAL)
			}
			SILENT => {
				self.db_handler.set_silent(user_id, true).await?;

				CreateInteractionResponseMessage::new()
					.content("I won't respond to your messages but will still count 'x3's")
					.flags(InteractionResponseFlags::EPHEMERAL)
			}
			VERBOSE => {
				self.db_handler.set_silent(user_id, false).await?;
				CreateInteractionResponseMessage::new()
					.content("I will now respond to your messages")
					.flags(InteractionResponseFlags::EPHEMERAL)
			}
			COUNTS => {
				let counts = self.db_handler.get_user_counts(user_id).await?;
				let content = match counts.as_slice() {
					&[] => "You don't have any 'x3's yet :c".to_owned(),
					counts => format!(
						"Here are your counts:\n{}",
						counts
							.iter()
							.map(|c| format!("{} - {}", c.emote, c.count))
							.collect::<Vec<_>>()
							.join("\n")
					),
				};
				CreateInteractionResponseMessage::new().content(content)
			}
			LEADERBOARD => {
				let Some(server_id) = cmd.guild_id else {
					return Ok(None);
				};
				let server_id = server_id.get();

				let leaderboard = match cmd.data.options.as_slice() {
					[] => self.db_handler.leaderboard(server_id, 3).await?,
					[arg, ..] => {
						if let CommandDataOptionValue::Integer(count) = arg.value {
							self.db_handler.leaderboard(server_id, count).await?
						} else {
							eprintln!("Argument {} has incorrect type", arg.name);
							return Ok(None);
						}
					}
				};
				let mut emote_map: HashMap<Box<str>, Vec<LeaderboardRow>> = HashMap::new();
				for row in leaderboard {
					match emote_map.entry(row.emote.clone()) {
						Entry::Occupied(mut o) => o.get_mut().push(row),
						Entry::Vacant(v) => v.insert(Default::default()).push(row),
					}
				}
				let content = emote_map
					.into_iter()
					.map(|(emote, rows)| {
						let rows_str = rows
							.iter()
							.map(ToString::to_string)
							.collect::<Box<[_]>>()
							.join("\n");
						format!("## {emote}\n{rows_str}")
					})
					.collect::<Box<[_]>>()
					.join("\n");

				CreateInteractionResponseMessage::new()
					.content(content)
					.allowed_mentions(CreateAllowedMentions::new())
			}
			MUTE_ALL => {
				let Some(server_id) = cmd.guild_id else {
					return Ok(None);
				};
				let server_id = server_id.get();

				self.db_handler.set_mute_all(server_id, true).await?;
				CreateInteractionResponseMessage::new().content(
					"I won't respond to messages in this server but will still count 'x3's",
				)
			}
			UNMUTE_ALL => {
				let Some(server_id) = cmd.guild_id else {
					return Ok(None);
				};
				let server_id = server_id.get();

				self.db_handler.set_mute_all(server_id, false).await?;
				CreateInteractionResponseMessage::new()
					.content("I will now respond to messages in this server")
			}
			_ => return Ok(None),
		};

		Ok(Some(CreateInteractionResponse::Message(msg)))
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

			match self.db_handler.is_silent(author_id, server_id).await {
				Ok(true) => return,
				Err(why) => return eprintln!("DB error: {why}"),
				_ => (),
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
	// env
	dotenv().ok();
	let token = env::var("BOT_TOKEN").expect("Expected a BOT_TOKEN in the environment");
	let db_url = env::var("DATABASE_URL").expect("Expected a DATABASE_URL in the environment");

	// db
	let pool = Pool::connect(&db_url).await.unwrap();
	let db_handler = DatabaseHandler::new(pool);

	// regex
	let regex = Regex::new(r#"([:;xX]3)3*c*$"#).unwrap();

	// intents
	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	// event handler
	let handler = Handler { regex, db_handler };

	// client
	let mut client = Client::builder(&token, intents)
		.event_handler(handler)
		.await
		.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {why:?}");
	}
}
