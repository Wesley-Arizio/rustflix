use core_database::entities::movies::MovieDAO;

#[derive(Debug)]
pub struct MovieDTO {
    pub id: String,
    pub title: String,
    pub description: String,
}

impl From<MovieDAO> for MovieDTO {
    fn from(value: MovieDAO) -> Self {
        Self {
            id: value.id.to_string(),
            title: value.title,
            description: value.description,
        }
    }
}
