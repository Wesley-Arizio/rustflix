use core_database::{
    entities::users::UserDAO,
    types::{DateTime, Utc, Uuid},
};

#[derive(Debug)]
pub struct UserDTO {
    pub id: Uuid,
    pub name: String,
    pub birthday: DateTime<Utc>,
    pub active: bool,
}

impl From<UserDAO> for UserDTO {
    fn from(value: UserDAO) -> Self {
        Self {
            id: value.id,
            name: value.name,
            birthday: value.birthday,
            active: value.active,
        }
    }
}
