use crate::input::PaginateInput;
use crate::Context;
use core::{dto::movie::MovieDTO, service::Core};
use database::types::Uuid;
use juniper::{graphql_object, FieldError, FieldResult, Value};
use std::str::FromStr;

pub struct QueryRoot {
    pub core: Core,
}

#[derive(Debug)]
pub struct Movie {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[graphql_object]
impl Movie {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn description(&self) -> &str {
        &self.description
    }
}

impl From<MovieDTO> for Movie {
    fn from(value: MovieDTO) -> Self {
        Self {
            id: value.id,
            title: value.title,
            description: value.description,
        }
    }
}

impl QueryRoot {
    pub fn new(core: Core) -> Self {
        Self { core }
    }
}

#[graphql_object(context = Context)]
impl QueryRoot {
    async fn movies(&self, ctx: &Context, input: PaginateInput) -> FieldResult<Vec<Movie>> {
        if let Some(session) = &ctx.session {
            if let Ok(Some(cookie)) = session.get::<String>("sid") {
                let movies = self
                    .core
                    .list_movies(cookie, input.offset as u32, input.limit as u32)
                    .await?
                    .into_iter()
                    .map(Movie::from)
                    .collect::<Vec<Movie>>();
                Ok(movies)
            } else {
                Err(FieldError::new("Invalid Credentials", Value::Null))
            }
        } else {
            eprintln!("cannot retrieve session from context");
            Err(FieldError::new("Internal Server Error", Value::Null))
        }
    }

    async fn movie(&self, ctx: &Context, movie_id: String) -> FieldResult<Option<Movie>> {
        if let Some(session) = &ctx.session {
            if let Ok(Some(cookie)) = session.get::<String>("sid") {
                let uuid = Uuid::from_str(&movie_id).unwrap();
                let movie = self.core.movie(cookie, uuid).await?.map(Movie::from);
                Ok(movie)
            } else {
                Err(FieldError::new("Invalid Credentials", Value::Null))
            }
        } else {
            eprintln!("cannot retrieve session from context");
            Err(FieldError::new("Internal Server Error", Value::Null))
        }
    }
}
