use sqlx::{Pool, Postgres};

pub struct DatabaseHandler {
	pool: Pool<Postgres>,
}
impl DatabaseHandler {
	pub fn new(pool: Pool<Postgres>) -> Self {
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
}
