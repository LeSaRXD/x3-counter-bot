use std::{fmt::Display, num::NonZeroU32};

use sqlx::PgPool;

pub struct UserCount {
	pub emote: Box<str>,
	pub count: i64,
}

#[derive(Debug)]
pub struct LeaderboardRow {
	pub emote: Box<str>,
	pub user_id: Box<str>,
	pub count: i64,
	pub rank: i64,
}
impl Display for LeaderboardRow {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}\\. <@{}> - {}", self.rank, self.user_id, self.count)
	}
}

pub enum VerboseLevel {
	Verbose,
	Every(NonZeroU32),
	Silent,
}
impl From<Option<i32>> for VerboseLevel {
	fn from(value: Option<i32>) -> Self {
		match value.map(|v| NonZeroU32::new(v as u32)) {
			Some(Some(nonzero)) => Self::Every(nonzero),
			Some(None) => Self::Silent,
			None => Self::Verbose,
		}
	}
}

pub struct DatabaseHandler {
	pool: PgPool,
}
impl DatabaseHandler {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}
}

impl DatabaseHandler {
	pub async fn add_one(
		&self,
		user_id: u64,
		server_id: u64,
		emote: &str,
	) -> sqlx::Result<Option<u32>> {
		if self.is_opt_out(user_id).await? {
			return Ok(None);
		}
		Ok(Some(
			query!(
				r#"INSERT INTO counter (user_id, server_id, emote, count) VALUES ($1, $2, $3,
					(SELECT COALESCE((SELECT count FROM counter WHERE user_id=$1 AND server_id = $2 AND emote=$3), 0) + 1)
				)
				ON CONFLICT (user_id, server_id, emote) DO
				UPDATE SET count = EXCLUDED.count
				RETURNING count"#,
				user_id.to_string(),
				server_id.to_string(),
				emote.to_lowercase(),
			)
			.fetch_one(&self.pool)
			.await?
			.count as u32,
		))
	}
	pub async fn get_user_counts(&self, user_id: u64) -> sqlx::Result<Vec<UserCount>> {
		sqlx::query_as!(
			UserCount,
			r#"SELECT emote, SUM(count) as "count!" FROM counter WHERE user_id = $1 GROUP BY emote"#,
			user_id.to_string()
		)
		.fetch_all(&self.pool)
		.await
	}

	pub async fn set_opt_out(&self, user_id: u64, value: bool) -> sqlx::Result<()> {
		sqlx::query!(
			r#"INSERT INTO options (user_id, opt_out) VALUES ($1, $2)
			ON CONFLICT (user_id) DO UPDATE
			SET opt_out = EXCLUDED.opt_out"#,
			user_id.to_string(),
			value,
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn is_opt_out(&self, user_id: u64) -> sqlx::Result<bool> {
		sqlx::query_scalar!(
			r#"SELECT EXISTS(
				SELECT * FROM options WHERE user_id = $1 AND opt_out
			) AS "exists!""#,
			user_id.to_string(),
		)
		.fetch_one(&self.pool)
		.await
	}

	pub async fn set_mute_all(&self, server_id: u64, value: bool) -> sqlx::Result<()> {
		sqlx::query!(
			r#"INSERT INTO server_options (server_id, mute_all) VALUES ($1, $2)
		ON CONFLICT (server_id) DO UPDATE
		SET mute_all = EXCLUDED.mute_all"#,
			server_id.to_string(),
			value,
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn set_silent(&self, user_id: u64, value: Option<u32>) -> sqlx::Result<()> {
		sqlx::query!(
			r#"INSERT INTO options (user_id, silent) VALUES ($1, $2)
			ON CONFLICT (user_id) DO UPDATE
			SET silent = EXCLUDED.silent"#,
			user_id.to_string(),
			value.map(|v| v as i32),
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn verbose_level(&self, user_id: u64, server_id: u64) -> sqlx::Result<VerboseLevel> {
		sqlx::query_scalar!(
			r#"SELECT CASE
				WHEN EXISTS(
					SELECT * FROM server_options WHERE server_id = $1 AND mute_all
				) THEN 0
				ELSE (SELECT silent FROM options WHERE user_id = $2)
			END"#,
			server_id.to_string(),
			user_id.to_string(),
		)
		.fetch_one(&self.pool)
		.await
		.map(Into::into)
	}

	pub async fn leaderboard(&self, server_id: u64, top: i64) -> sqlx::Result<Vec<LeaderboardRow>> {
		sqlx::query_as!(
			LeaderboardRow,
			r#"WITH ranked AS (
				SELECT user_id, emote, count,
				DENSE_RANK() OVER (PARTITION BY emote ORDER BY count DESC) AS rank
				FROM counter WHERE server_id = $1
			)
			SELECT
				emote,
				user_id,
				count,
				rank AS "rank!"
			FROM ranked
			WHERE rank <= $2
			ORDER BY emote, rank ASC"#,
			server_id.to_string(),
			top as i64,
		)
		.fetch_all(&self.pool)
		.await
	}
}
