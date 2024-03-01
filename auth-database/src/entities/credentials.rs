use crate::{
    connection::{Pool, Postgres},
    traits::{DatabaseError, EntityRepository},
    types::Uuid,
};

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CredentialsDAO {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub active: bool,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CreateCredentialsDAO {
    pub email: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UpdateCredentialsDAO {
    pub password: String,
    pub active: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialsBy {
    Id(Uuid),
    Email(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CredentialsWhere {
    Active(bool),
}

#[derive(Debug)]
pub struct CredentialsRepository;

#[async_trait::async_trait]
impl
    EntityRepository<
        Postgres,
        CredentialsDAO,
        CreateCredentialsDAO,
        UpdateCredentialsDAO,
        CredentialsBy,
        CredentialsWhere,
    > for CredentialsRepository
{
    async fn insert(
        db: &Pool<Postgres>,
        input: CreateCredentialsDAO,
    ) -> Result<CredentialsDAO, DatabaseError> {
        sqlx::query_as::<_, CredentialsDAO>("INSERT INTO credentials (email, password) VALUES ($1, $2) RETURNING id, email, password, active;")
            .bind(input.email)
            .bind(input.password)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn delete(
        db: &Pool<Postgres>,
        key: CredentialsBy,
    ) -> Result<CredentialsDAO, DatabaseError> {
        match key {
            CredentialsBy::Id(uuid) => {
                sqlx::query_as::<_, CredentialsDAO>("UPDATE credentials SET active = false WHERE id = $1 RETURNING id, email, password, active;")
                    .bind(uuid)
                    .fetch_one(db)
                    .await
                    .map_err(DatabaseError::from)
            },
            CredentialsBy::Email(email) => {
                sqlx::query_as::<_, CredentialsDAO>("UPDATE credentials SET active = false WHERE email = $1 RETURNING id, password, email, active;")
                    .bind(email)
                    .fetch_one(db)
                    .await
                    .map_err(DatabaseError::from)
            },

        }
    }

    async fn update(
        db: &Pool<Postgres>,
        key: CredentialsBy,
        update: UpdateCredentialsDAO,
    ) -> Result<CredentialsDAO, DatabaseError> {
        match key {
            CredentialsBy::Id(id) => sqlx::query_as::<_, CredentialsDAO>(
                "UPDATE credentials SET password = $2, active = $3 WHERE id = $1 RETURNING id, email, password, active;",
            )
            .bind(id)
                .bind(update.password)
            .bind(update.active)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
            CredentialsBy::Email(email) => sqlx::query_as::<_, CredentialsDAO>(
                "UPDATE credentials SET password = $2, active = $3 WHERE email = $1 RETURNING id, email, password, active;",
            )
                .bind(email)
                .bind(update.password)
                .bind(update.active)
                .fetch_one(db)
                .await
                .map_err(DatabaseError::from),
        }
    }

    async fn get(db: &Pool<Postgres>, key: CredentialsBy) -> Result<CredentialsDAO, DatabaseError> {
        match key {
            CredentialsBy::Id(id) => sqlx::query_as::<_, CredentialsDAO>(
                "SELECT id, email, password, active FROM credentials WHERE id = $1 LIMIT 1;",
            )
            .bind(id)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
            CredentialsBy::Email(email) => sqlx::query_as::<_, CredentialsDAO>(
                "SELECT id, email, password, active FROM credentials WHERE email = $1 LIMIT 1;",
            )
            .bind(email)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
        }
    }

    async fn try_get(
        db: &Pool<Postgres>,
        key: CredentialsBy,
    ) -> Result<Option<CredentialsDAO>, DatabaseError> {
        match key {
            CredentialsBy::Id(uuid) => {
                sqlx::query_as("SELECT id, email, password, active FROM credentials WHERE id = $1;")
                    .bind(uuid)
                    .fetch_optional(db)
                    .await
                    .map_err(DatabaseError::from)
            }
            CredentialsBy::Email(email) => sqlx::query_as(
                "SELECT id, email, password, active FROM credentials WHERE email = $1;",
            )
            .bind(email)
            .fetch_optional(db)
            .await
            .map_err(DatabaseError::from),
        }
    }
}

#[cfg(feature = "integration")]
#[cfg(test)]
mod tests {
    use crate::connection::PgPool;
    use crate::entities::credentials::{
        CreateCredentialsDAO, CredentialsBy, CredentialsRepository, UpdateCredentialsDAO,
    };
    use crate::traits::EntityRepository;
    use dotenv;

    #[tokio::test]
    async fn test_db() {
        dotenv::dotenv().ok();
        let url =
            std::env::var("TEST_AUTH_DATABASE_URL").expect("TEST_AUTH_DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await.unwrap();

        // create credential
        let response = CredentialsRepository::insert(
            &pool,
            CreateCredentialsDAO {
                email: "akira".to_string(),
                password: String::from("password"),
            },
        )
        .await
        .expect("Could not create credential");

        // get user
        let found = CredentialsRepository::get(&pool, CredentialsBy::Id(response.id))
            .await
            .expect("User not found");

        assert_eq!(response.id, found.id);
        assert_eq!(response.email, found.email);
        assert_eq!(response.password, found.password);
        assert!(response.active);

        // try_get user, returns none if user isn't found
        let found = CredentialsRepository::try_get(&pool, CredentialsBy::Id(response.id))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(response.id, found.id);
        assert_eq!(response.email, found.email);
        assert_eq!(response.password, found.password);
        assert!(found.active);

        // update
        let updated = CredentialsRepository::update(
            &pool,
            CredentialsBy::Id(response.id),
            UpdateCredentialsDAO {
                password: "other password".to_string(),
                active: true,
            },
        )
        .await
        .expect("Could not update user");

        assert_eq!(response.id, updated.id);
        assert_eq!(updated.password, "other password");
        assert!(updated.active);

        // delete
        let deleted = CredentialsRepository::delete(&pool, CredentialsBy::Id(response.id))
            .await
            .expect("Could not delete an user");
        assert_eq!(response.id, deleted.id);
        assert!(!deleted.active);
    }
}
