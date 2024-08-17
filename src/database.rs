use std::fmt::Display;

use sqlx::PgPool;

pub struct UserCount {
	pub emote: Box<str>,
	pub count: i32,
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

pub struct DatabaseHandler {
	pool: PgPool,
}
impl DatabaseHandler {
	pub fn new(pool: PgPool) -> Self {
		Self { pool }
	}
}

impl DatabaseHandler {
	pub async fn add_one(&self, user_id: u64, emote: &str) -> sqlx::Result<Option<i32>> {
		if self.is_opt_out(user_id).await? {
			return Ok(None);
		}
		Ok(Some(
			query!(
				r#"INSERT INTO counter (user_id, emote, count) VALUES ($1, $2, 
					(SELECT COALESCE((SELECT count FROM counter WHERE user_id=$1 and emote=$2), 0) + 1)
				)
				ON CONFLICT (user_id, emote) DO
				UPDATE SET count = EXCLUDED.count
				RETURNING count"#,
				user_id.to_string(),
				emote.to_lowercase(),
			)
			.fetch_one(&self.pool)
			.await?
			.count,
		))
	}
	pub async fn get_user_counts(&self, user_id: u64) -> sqlx::Result<Vec<UserCount>> {
		sqlx::query_as!(
			UserCount,
			r#"SELECT emote, count FROM counter WHERE user_id = $1"#,
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

	pub async fn set_silent(&self, user_id: u64, value: bool) -> sqlx::Result<()> {
		sqlx::query!(
			r#"INSERT INTO options (user_id, silent) VALUES ($1, $2)
			ON CONFLICT (user_id) DO UPDATE
			SET silent = EXCLUDED.silent"#,
			user_id.to_string(),
			value,
		)
		.execute(&self.pool)
		.await
		.map(|_| ())
	}
	pub async fn is_silent(&self, user_id: u64) -> sqlx::Result<bool> {
		sqlx::query_scalar!(
			r#"SELECT EXISTS(
				SELECT * FROM options WHERE user_id = $1 AND silent
			) AS "exists!""#,
			user_id.to_string(),
		)
		.fetch_one(&self.pool)
		.await
	}

	pub async fn leaderboard(&self, top: u64) -> sqlx::Result<Vec<LeaderboardRow>> {
		sqlx::query_as!(
			LeaderboardRow,
			r#"WITH ranked AS (
				SELECT user_id, emote, count, DENSE_RANK() OVER (PARTITION BY emote ORDER BY count DESC) AS rank FROM counter
			)
			SELECT
				emote,
				user_id,
				count,
				rank AS "rank!"
			FROM ranked
			WHERE rank <= $1
			ORDER BY emote, rank ASC"#,
			top as i64,
		)
		.fetch_all(&self.pool)
		.await
	}
}
