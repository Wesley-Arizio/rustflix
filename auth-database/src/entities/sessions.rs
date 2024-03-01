use crate::{
    connection::{Pool, Postgres},
    traits::{DatabaseError, EntityRepository},
    types::{DateTime, Utc, Uuid},
};

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct SessionsDAO {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub credential_id: Uuid,
    pub active: bool,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UpdateSessionsDAO {}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionsBy {
    Id(Uuid),
    CredentialId(Uuid),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionsWhere {
    CredentialId(Uuid),
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CreateSessionsDAO {
    pub expires_at: DateTime<Utc>,
    pub credential_id: Uuid,
}

#[derive(Debug)]
pub struct SessionsRepository;

#[async_trait::async_trait]
impl
    EntityRepository<
        Postgres,
        SessionsDAO,
        CreateSessionsDAO,
        UpdateSessionsDAO,
        SessionsBy,
        SessionsWhere,
    > for SessionsRepository
{
    async fn insert(
        db: &Pool<Postgres>,
        input: CreateSessionsDAO,
    ) -> Result<SessionsDAO, DatabaseError> {
        sqlx::query_as::<_, SessionsDAO>("INSERT INTO sessions (expires_at, credential_id) VALUES ($1, $2) RETURNING id, created_at, expires_at, credential_id, active;")
            .bind(input.expires_at)
            .bind(input.credential_id)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn delete(db: &Pool<Postgres>, key: SessionsBy) -> Result<SessionsDAO, DatabaseError> {
        match key {
            SessionsBy::Id(uuid) => {
                sqlx::query_as::<_, SessionsDAO>("UPDATE sessions SET active = false WHERE id = $1 RETURNING id, created_at, expires_at, credential_id, active;")
                    .bind(uuid)
                    .fetch_one(db)
                    .await
                    .map_err(DatabaseError::from)
            },
            SessionsBy::CredentialId(uuid) => {
                sqlx::query_as::<_, SessionsDAO>("UPDATE sessions SET active = false WHERE credential_id = $1 RETURNING id, created_at, expires_at, credential_id, active;")
                    .bind(uuid)
                    .fetch_one(db)
                    .await
                    .map_err(DatabaseError::from)
            },

        }
    }

    async fn update(
        _db: &Pool<Postgres>,
        _key: SessionsBy,
        _update: UpdateSessionsDAO,
    ) -> Result<SessionsDAO, DatabaseError> {
        unreachable!("")
    }

    async fn get(db: &Pool<Postgres>, key: SessionsBy) -> Result<SessionsDAO, DatabaseError> {
        match key {
            SessionsBy::Id(id) => sqlx::query_as::<_, SessionsDAO>(
                "SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE id = $1 LIMIT 1;",
            )
            .bind(id)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
            SessionsBy::CredentialId(uuid) => sqlx::query_as::<_, SessionsDAO>(
                "SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE credential_id = $1 LIMIT 1;",
            )
                .bind(uuid)
                .fetch_one(db)
                .await
                .map_err(DatabaseError::from),
        }
    }

    async fn try_get(
        db: &Pool<Postgres>,
        key: SessionsBy,
    ) -> Result<Option<SessionsDAO>, DatabaseError> {
        match key {
            SessionsBy::Id(uuid) => {
                sqlx::query_as("SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE id = $1 LIMIT 1;")
                    .bind(uuid)
                    .fetch_optional(db)
                    .await
                    .map_err(DatabaseError::from)
            },
            SessionsBy::CredentialId(uuid) => {
                sqlx::query_as("SELECT id, created_at, expires_at, credential_id, active FROM sessions WHERE credential_id = $1 LIMIT 1;")
                    .bind(uuid)
                    .fetch_optional(db)
                    .await
                    .map_err(DatabaseError::from)
            }
        }
    }
}

#[cfg(feature = "integration")]
#[cfg(test)]
mod tests {
    use crate::connection::PgPool;
    use crate::entities::credentials::{
        CredentialsBy, CredentialsDAO, CredentialsRepository, UpdateCredentialsDAO,
    };
    use crate::entities::sessions::{
        CreateSessionsDAO, SessionsBy, SessionsDAO, SessionsRepository,
    };
    use crate::traits::EntityRepository;
    use database::types::Utc;
    use dotenv;
    use sqlx::types::uuid::Uuid;
    use std::time::Duration;

    #[tokio::test]
    async fn test_db() {
        dotenv::dotenv().ok();
        let url =
            std::env::var("TEST_AUTH_DATABASE_URL").expect("TEST_AUTH_DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await.unwrap();

        // create credential
        let response = CredentialsRepository::insert(
            &pool,
            CredentialsDAO {
                id: Uuid::new_v4(),
                email: "akira".to_string(),
                password: String::from("password"),
                active: true,
            },
        )
        .await
        .expect("Could not create credential");

        let session = SessionsRepository::insert(
            &pool,
            CreateSessionsDAO {
                expires_at: Utc::now() + Duration::from_secs(60 * 5),
                credential_id: response.id,
            },
        )
        .await
        .expect("Could not create session");

        assert_eq!(session.credential_id, response.id);
        assert!(session.active);

        // get session
        let found = SessionsRepository::get(&pool, SessionsBy::Id(session.id))
            .await
            .expect("User not found");

        assert_eq!(session.id, found.id);
        assert!(session.expires_at.timestamp() > session.created_at.timestamp());
        assert!(found.active);

        // try_get credential, returns none if one isn't found
        let found = SessionsRepository::try_get(&pool, SessionsBy::Id(session.id))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(session.id, found.id);

        // delete
        let deleted = SessionsRepository::delete(&pool, SessionsBy::Id(session.id))
            .await
            .expect("Could not delete a session");
        assert_eq!(session.id, deleted.id);
        assert!(!deleted.active);
    }
}
