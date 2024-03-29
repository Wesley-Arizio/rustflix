use crate::{
    connection::{Pool, Postgres},
    traits::{DatabaseError, EntityRepository},
    types::Uuid,
};

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct MovieDAO {
    pub id: Uuid,
    pub title: String,
    pub description: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct CreateMovieDAO {
    pub title: String,
    pub description: String,
}

#[derive(sqlx::FromRow, Debug, PartialEq, Eq, Clone)]
pub struct UpdateMovieDAO {
    pub title: String,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MovieBy {
    Id(Uuid),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MoviesWhere {
    Page { offset: u32, limit: u32 },
}

#[derive(Debug)]
pub struct MovieRepository;

#[async_trait::async_trait]
impl EntityRepository<Postgres, MovieDAO, CreateMovieDAO, UpdateMovieDAO, MovieBy, MoviesWhere>
    for MovieRepository
{
    async fn insert(db: &Pool<Postgres>, input: CreateMovieDAO) -> Result<MovieDAO, DatabaseError> {
        sqlx::query_as::<_, MovieDAO>("INSERT INTO movies (title, description) VALUES ($1, $2) RETURNING id, title, description;")
            .bind(input.title)
            .bind(input.description)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn delete(db: &Pool<Postgres>, key: MovieBy) -> Result<MovieDAO, DatabaseError> {
        match key {
            MovieBy::Id(uuid) => sqlx::query_as::<_, MovieDAO>(
                "DELETE FROM movies WHERE id = $1 RETURNING id, title, description;",
            )
            .bind(uuid)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
        }
    }

    async fn update(
        db: &Pool<Postgres>,
        key: MovieBy,
        update: UpdateMovieDAO,
    ) -> Result<MovieDAO, DatabaseError> {
        match key {
            MovieBy::Id(uuid) => sqlx::query_as::<_, MovieDAO>(
                "UPDATE movies SET title = $1, description = $2 WHERE id = $3 RETURNING id, title, description;",
            )
                .bind(update.title)
                .bind(update.description)
                .bind(uuid)
                .fetch_one(db)
                .await
                .map_err(DatabaseError::from),
        }
    }

    async fn get(db: &Pool<Postgres>, key: MovieBy) -> Result<MovieDAO, DatabaseError> {
        match key {
            MovieBy::Id(uuid) => sqlx::query_as::<_, MovieDAO>(
                "SELECT id, title, description FROM movies WHERE id = $1 LIMIT 1;",
            )
            .bind(uuid)
            .fetch_one(db)
            .await
            .map_err(DatabaseError::from),
        }
    }

    async fn try_get(db: &Pool<Postgres>, key: MovieBy) -> Result<Option<MovieDAO>, DatabaseError> {
        match key {
            MovieBy::Id(uuid) => {
                sqlx::query_as("SELECT id, title, description FROM movies WHERE id = $1;")
                    .bind(uuid)
                    .fetch_optional(db)
                    .await
                    .map_err(DatabaseError::from)
            }
        }
    }

    async fn get_all(
        db: &Pool<Postgres>,
        key: MoviesWhere,
    ) -> Result<Vec<MovieDAO>, DatabaseError> {
        match key {
            MoviesWhere::Page { offset, limit } => sqlx::query_as::<_, MovieDAO>(
                "SELECT id, title, description FROM movies OFFSET $1 LIMIT $2;",
            )
            .bind(offset as i32)
            .bind(limit as i32)
            .fetch_all(db)
            .await
            .map_err(DatabaseError::from),
        }
    }
}

#[cfg(feature = "integration")]
#[cfg(test)]
mod tests {
    use crate::connection::PgPool;
    use crate::entities::movies::{
        CreateMovieDAO, MovieBy, MovieRepository, MoviesWhere, UpdateMovieDAO,
    };
    use crate::traits::EntityRepository;
    use dotenv;
    #[tokio::test]
    async fn test_db() {
        dotenv::dotenv().ok();
        let url =
            std::env::var("TEST_CORE_DATABASE_URL").expect("TEST_CORE_DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await.unwrap();

        // create movie
        let response = MovieRepository::insert(
            &pool,
            CreateMovieDAO {
                title: "Avengers infinity war".to_string(),
                description: "crazy movie".to_string(),
            },
        )
        .await
        .expect("Could not create movie");

        // list movie
        let _ = MovieRepository::insert(
            &pool,
            CreateMovieDAO {
                title: "Doctor strange".to_string(),
                description: "crazy movie".to_string(),
            },
        )
        .await
        .expect("Could not create movie");

        let _ = MovieRepository::insert(
            &pool,
            CreateMovieDAO {
                title: "Spider man".to_string(),
                description: "crazy movie".to_string(),
            },
        )
        .await
        .expect("Could not create movie");

        let movies = MovieRepository::get_all(
            &pool,
            MoviesWhere::Page {
                offset: 0,
                limit: 2,
            },
        )
        .await
        .unwrap();
        assert_eq!(movies.len(), 2);
        assert_eq!(movies[0].title, "Avengers infinity war");
        assert_eq!(movies[1].title, "Doctor strange");

        let movies = MovieRepository::get_all(
            &pool,
            MoviesWhere::Page {
                offset: 2,
                limit: 2,
            },
        )
        .await
        .unwrap();
        assert_eq!(movies.len(), 1);
        assert_eq!(movies[0].title, "Spider man");

        // get movie
        let found = MovieRepository::get(&pool, MovieBy::Id(response.id))
            .await
            .expect("Movie not found");

        assert_eq!(response.id, found.id);
        assert_eq!(response.title, found.title);
        assert_eq!(response.description, found.description);

        // try_get movie, returns none if movie isn't found
        let found = MovieRepository::try_get(&pool, MovieBy::Id(response.id))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(response.id, found.id);
        assert_eq!(response.title, found.title);
        assert_eq!(response.description, found.description);

        // update
        let updated = MovieRepository::update(
            &pool,
            MovieBy::Id(response.id),
            UpdateMovieDAO {
                title: "Avengers endgame".to_string(),
                description: "best movie".to_string(),
            },
        )
        .await
        .expect("Could not update movie");

        assert_eq!(response.id, updated.id);
        assert_eq!(updated.title, "Avengers endgame");
        assert_eq!(updated.description, "best movie");

        // delete
        let deleted = MovieRepository::delete(&pool, MovieBy::Id(response.id))
            .await
            .expect("Could not delete an movie");
        assert_eq!(response.id, deleted.id);
    }
}
