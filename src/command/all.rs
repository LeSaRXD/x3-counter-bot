use std::collections::{hash_map::Entry, HashMap};

use serenity::all::{
	CommandDataOptionValue, CommandInteraction, CreateAllowedMentions,
	CreateInteractionResponseMessage, Permissions,
};

use crate::{
	arg,
	command::command,
	database::{DatabaseHandler, LeaderboardRow},
};

use super::args::{IntArg, IntoCommandArg, UserArg};

fn postfix(count: i64) -> &'static str {
	match count % 10 {
		2 => "nd",
		3 => "rd",
		_ => "th",
	}
}

macro_rules! response {
	(server error) => {
		response!("You can only run this command in a server (this should not be possible)")
	};
	(argument error) => {
		response!("Incorrect argument type provided (this should not be possible)")
	};
	($res:expr) => {
		Ok(CreateInteractionResponseMessage::new()
			.content($res)
			.ephemeral(true)
			.allowed_mentions(CreateAllowedMentions::new()))
	};
	($res:expr, false) => {
		Ok(CreateInteractionResponseMessage::new()
			.content($res)
			.ephemeral(false)
			.allowed_mentions(CreateAllowedMentions::new()))
	};
}

const REPEAT_ARG: IntArg = arg!(
	Int,
	"send_on",
	"Send counts every nth message",
	false,
	Some(2),
	Some(i32::MAX as u64)
);

const LEADERBOARD_COUNT_ARG: IntArg = arg!(
	Int,
	"count",
	"Max count of users per emote",
	false,
	Some(1),
	Some(5)
);

const COUNTS_USER_ARG: UserArg = arg!(User, "user", "The user whose counts to display", false);

command!(OptInCommand, "opt_in", "Start tracking x3s");
pub async fn opt_in(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let user_id = cmd.user.id.get();

	db.set_opt_out(user_id, false).await?;
	response!("I will count your ':3's now UwU")
}

command!(OptOutCommand, "opt_out", "Stop tracking x3s");
pub async fn opt_out(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let user_id = cmd.user.id.get();

	db.set_opt_out(user_id, true).await?;
	response!("I won't count your ':3's now qwq")
}

command!(
	SilentCommand,
	"silent",
	"Track x3s silently (don't send messages)",
	[REPEAT_ARG]
);
pub async fn silent(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let user_id = cmd.user.id.get();

	let content = match cmd.data.options.as_slice() {
		[] => {
			db.set_silent(user_id, Some(0)).await?;
			"I won't respond to your messages but will still count x3s".to_owned()
		}
		[arg, ..] => {
			if let CommandDataOptionValue::Integer(count) = arg.value {
				db.set_silent(user_id, Some(count as u32)).await?;

				format!(
					"I will only respond to every {}{} x3",
					count,
					postfix(count)
				)
			} else {
				eprintln!("Argument {} has incorrect type", arg.name);
				return response!(argument error);
			}
		}
	};

	response!(content)
}

command!(
	VerboseCommand,
	"verbose",
	"Track x3s verbosely (do send messages)"
);
pub async fn verbose(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let user_id = cmd.user.id.get();

	db.set_silent(user_id, None).await?;
	response!("I will now respond to your messages")
}

command!(
	CountsCommand,
	"counts",
	"Get your x3 counts",
	[COUNTS_USER_ARG]
);
pub async fn counts(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let user_id = cmd.user.id.get();

	let user_arg = match cmd.data.options.as_slice() {
		[] => None,
		[arg, ..] => {
			if let CommandDataOptionValue::User(user_arg_id) = arg.value {
				Some(user_arg_id.get())
			} else {
				eprintln!("Argument {} has incorrect type", arg.name);
				return response!(argument error);
			}
		}
	};

	let query_user_id = user_arg.unwrap_or(user_id);

	let counts = match cmd.guild_id {
		Some(server_id) => {
			db.get_user_server_counts(query_user_id, server_id.get())
				.await?
		}
		None => db.get_user_counts(user_id).await?,
	};

	let content = match (counts.as_slice(), user_id == query_user_id) {
		(&[], true) => "You don't have any x3s yet :c".to_owned(),
		(&[], false) => "This user doesn't have any x3s yet :c".to_owned(),
		(counts, is_user) => {
			let counts_str = counts
				.iter()
				.map(|c| format!("{} - {}", c.emote, c.count))
				.collect::<Vec<_>>()
				.join("\n");
			if is_user {
				format!("Here are your counts:\n{counts_str}")
			} else {
				format!("Here are <@{query_user_id}>'s counts:\n{counts_str}")
			}
		}
	};
	response!(content, false)
}

command!(
	LeaderboardCommand,
	"leaderboard",
	"Get the x3 leaderboard for this server",
	false,
	[LEADERBOARD_COUNT_ARG]
);
pub async fn leaderboard(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let Some(server_id) = cmd.guild_id else {
		return response!(server error);
	};
	let server_id = server_id.get();

	let leaderboard = match cmd.data.options.as_slice() {
		[] => db.leaderboard(server_id, 3i64).await?,
		[arg, ..] => {
			if let CommandDataOptionValue::Integer(count) = arg.value {
				db.leaderboard(server_id, count).await?
			} else {
				eprintln!("Argument {} has incorrect type", arg.name);
				return response!(argument error);
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

	response!(content, false)
}

command!(
	MuteAllCommand,
	"mute_all",
	"Mute all count messages in the server",
	false,
	Permissions::MANAGE_MESSAGES,
	[REPEAT_ARG]
);
pub async fn mute_all(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let Some(server_id) = cmd.guild_id else {
		return response!(server error);
	};
	let server_id = server_id.get();

	let content = match cmd.data.options.as_slice() {
		[] => {
			db.mute_all(server_id, Some(0)).await?;
			"I won't respond to messages in this server but will still count x3s".to_owned()
		}
		[arg, ..] => {
			if let CommandDataOptionValue::Integer(count) = arg.value {
				db.mute_all(server_id, Some(count as u32)).await?;

				format!(
					"I will only respond to every {}{} x3 in this server",
					count,
					postfix(count)
				)
			} else {
				eprintln!("Argument {} has incorrect type", arg.name);
				return response!(argument error);
			}
		}
	};

	response!(content)
}

command!(
	UnmuteAllCommand,
	"unmute_all",
	"Unmute all count messages in the server",
	false,
	Permissions::MANAGE_MESSAGES
);
pub async fn unmute_all(
	db: &DatabaseHandler,
	cmd: &CommandInteraction,
) -> sqlx::Result<CreateInteractionResponseMessage> {
	let Some(server_id) = cmd.guild_id else {
		return response!(server error);
	};
	let server_id = server_id.get();

	db.mute_all(server_id, None).await?;
	response!("I will now respond to messages in this server")
}
