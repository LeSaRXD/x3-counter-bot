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
	pub async fn set_opt_out(&self, user_id: u64, value: bool) -> sqlx::Result<()> {
		if value {
			sqlx::query!(
				r#"INSERT INTO opt_out (user_id) VALUES ($1)
				ON CONFLICT (user_id) DO NOTHING"#,
				user_id.to_string(),
			)
			.execute(&self.pool)
			.await?;
		} else {
			sqlx::query!(
				r#"DELETE FROM opt_out WHERE user_id=$1"#,
				user_id.to_string(),
			)
			.execute(&self.pool)
			.await?;
		}
		Ok(())
	}

	pub async fn is_opted_out(&self, user_id: u64) -> sqlx::Result<bool> {
		sqlx::query!(
			r#"SELECT EXISTS(
				SELECT * FROM opt_out WHERE user_id=$1
			) AS "exists!""#,
			user_id.to_string(),
		)
		.fetch_one(&self.pool)
		.await
		.map(|r| r.exists)
	}

	pub async fn add_one(&self, user_id: u64, emote: &str) -> sqlx::Result<Option<i32>> {
		if self.is_opted_out(user_id).await? {
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
}
