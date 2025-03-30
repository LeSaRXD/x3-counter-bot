mod unsigned;

use std::{fmt::Display, num::NonZeroU32};

use sqlx::PgPool;
use unsigned::{PsqlU32, PsqlU64};

pub struct UserCount {
	pub emote: Box<str>,
	pub count: PsqlU64,
}

#[derive(Debug)]
pub struct LeaderboardRow {
	pub emote: Box<str>,
	pub user_id: PsqlU64,
	pub count: PsqlU32,
	pub rank: PsqlU64,
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
		user_id: impl Into<PsqlU64>,
		server_id: impl Into<PsqlU64>,
		emote: &str,
	) -> sqlx::Result<Option<u32>> {
		let user_id = user_id.into();
		let server_id = server_id.into();
		if self.is_opt_out(user_id).await? {
			return Ok(None);
		}
		Ok(Some(
			query_scalar!(
				r#"INSERT INTO counter (user_id, server_id, emote, count) VALUES ($1, $2, $3, 1)
				ON CONFLICT (user_id, server_id, emote) DO
				UPDATE SET count = counter.count + 1
				RETURNING count"#,
				i64::from(user_id),
				i64::from(server_id),
				emote,
			)
			.fetch_one(&self.pool)
			.await? as u32,
		))
	}
	pub async fn get_user_counts(
		&self,
		user_id: impl Into<PsqlU64>,
	) -> sqlx::Result<Vec<UserCount>> {
		let user_id = user_id.into();
		sqlx::query_as!(
			UserCount,
			r#"SELECT emote, SUM(count) as "count!" FROM counter WHERE user_id = $1 GROUP BY emote"#,
			i64::from(user_id)
		)
		.fetch_all(&self.pool)
		.await
	}
	pub async fn get_user_server_counts(
		&self,
		user_id: impl Into<PsqlU64>,
		server_id: impl Into<PsqlU64>,
	) -> sqlx::Result<Vec<UserCount>> {
		let user_id = user_id.into();
		let server_id = server_id.into();
		sqlx::query_as!(
			UserCount,
			r#"SELECT emote, SUM(count) as "count!" FROM counter WHERE user_id = $1 AND server_id = $2 GROUP BY emote"#,
			i64::from(user_id),
			i64::from(server_id),
		)
		.fetch_all(&self.pool)
		.await
	}

	pub async fn set_opt_out(&self, user_id: impl Into<PsqlU64>, value: bool) -> sqlx::Result<()> {
		let user_id = user_id.into();
		sqlx::query!(
			r#"INSERT INTO options (user_id, opt_out) VALUES ($1, $2)
			ON CONFLICT (user_id) DO UPDATE
			SET opt_out = EXCLUDED.opt_out"#,
			i64::from(user_id),
			value,
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn is_opt_out(&self, user_id: impl Into<PsqlU64>) -> sqlx::Result<bool> {
		let user_id = user_id.into();
		sqlx::query_scalar!(
			r#"SELECT EXISTS(
				SELECT * FROM options WHERE user_id = $1 AND opt_out
			) AS "exists!""#,
			i64::from(user_id),
		)
		.fetch_one(&self.pool)
		.await
	}

	pub async fn mute_all(
		&self,
		server_id: impl Into<PsqlU64>,
		value: Option<u32>,
	) -> sqlx::Result<()> {
		let server_id = server_id.into();
		sqlx::query!(
			r#"INSERT INTO server_options (server_id, mute_all) VALUES ($1, $2)
		ON CONFLICT (server_id) DO UPDATE
		SET mute_all = EXCLUDED.mute_all"#,
			i64::from(server_id),
			value.map(|v| v as i32),
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn set_silent(
		&self,
		user_id: impl Into<PsqlU64>,
		value: Option<u32>,
	) -> sqlx::Result<()> {
		let user_id = user_id.into();
		sqlx::query!(
			r#"INSERT INTO options (user_id, silent) VALUES ($1, $2)
			ON CONFLICT (user_id) DO UPDATE
			SET silent = EXCLUDED.silent"#,
			i64::from(user_id),
			value.map(|v| v as i32),
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn verbose_level(
		&self,
		user_id: impl Into<PsqlU64>,
		server_id: impl Into<PsqlU64>,
	) -> sqlx::Result<VerboseLevel> {
		let user_id = user_id.into();
		let server_id = server_id.into();
		sqlx::query_scalar!(
			r#"SELECT COALESCE((SELECT mute_all FROM server_options WHERE server_id = $1), (SELECT silent FROM options WHERE user_id = $2))"#,
			i64::from(server_id),
			i64::from(user_id),
		)
		.fetch_one(&self.pool)
		.await
		.map(Into::into)
	}

	pub async fn leaderboard(
		&self,
		server_id: impl Into<PsqlU64>,
		top: impl Into<PsqlU64>,
	) -> sqlx::Result<Vec<LeaderboardRow>> {
		let top = top.into();
		let server_id = server_id.into();
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
			i64::from(server_id),
			i64::from(top),
		)
		.fetch_all(&self.pool)
		.await
	}
}
