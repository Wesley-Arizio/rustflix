use crate::input::Birthday;
use core::dto::user::UserDTO;
use juniper::graphql_object;

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub birthday: Birthday,
}

#[graphql_object]
impl User {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn active(&self) -> bool {
        self.active
    }

    fn birthday(&self) -> &Birthday {
        &self.birthday
    }
}

impl From<UserDTO> for User {
    fn from(value: UserDTO) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.name,
            active: value.active,
            birthday: Birthday(value.birthday),
        }
    }
}
