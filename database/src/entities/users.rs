use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    Pool, Postgres,
};

use crate::traits::{DatabaseError, EntityRepository};

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UserDAO {
    pub id: Uuid,
    pub name: String,
    pub birthday: DateTime<Utc>,
    pub active: bool,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UpdateUserDAO {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UserBy {
    Id(Uuid),
}

#[derive(Debug, PartialEq, Eq)]
pub enum UsersWhere {}

#[derive(Debug)]
pub struct UserRepository;

#[async_trait::async_trait]
impl EntityRepository<Postgres, UserDAO, UserDAO, UpdateUserDAO, UserBy, UsersWhere>
    for UserRepository
{
    async fn insert(db: &Pool<Postgres>, input: UserDAO) -> Result<UserDAO, DatabaseError> {
        sqlx::query_as::<_, UserDAO>("INSERT INTO users (id, name, birthday) VALUES ($1, $2, $3) RETURNING id, name, birthday, active;")
            .bind(input.id)
            .bind(input.name)
            .bind(input.birthday)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn delete(db: &Pool<Postgres>, key: UserBy) -> Result<UserDAO, DatabaseError> {
        match key {
            UserBy::Id(uuid) => {
                sqlx::query_as::<_, UserDAO>("UPDATE users SET active = false WHERE id = $1 RETURNING id, name, active, birthday;")
                    .bind(uuid)
                    .fetch_one(db)
                    .await
                    .map_err(DatabaseError::from)
            }
        }
    }

    async fn update(
        db: &Pool<Postgres>,
        key: UserBy,
        update: UpdateUserDAO,
    ) -> Result<UserDAO, DatabaseError> {
        match key {
            UserBy::Id(uuid) => sqlx::query_as::<_, UserDAO>(
                "UPDATE users SET name = $1 WHERE id = $2 RETURNING id, name, active, birthday;",
            )
            .bind(update.name)
            .bind(uuid)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
        }
    }

    async fn get(db: &Pool<Postgres>, key: UserBy) -> Result<UserDAO, DatabaseError> {
        match key {
            UserBy::Id(uuid) => sqlx::query_as::<_, UserDAO>(
                "SELECT id, name, birthday, active FROM users WHERE id = $1 LIMIT 1;",
            )
            .bind(uuid)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
        }
    }

    async fn try_get(db: &Pool<Postgres>, key: UserBy) -> Result<Option<UserDAO>, DatabaseError> {
        match key {
            UserBy::Id(uuid) => {
                sqlx::query_as("SELECT id, name, birthday, active FROM users WHERE id = $1;")
                    .bind(uuid)
                    .fetch_optional(db)
                    .await
                    .map_err(DatabaseError::from)
            }
        }
    }
}
