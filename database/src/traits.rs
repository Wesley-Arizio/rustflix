use sqlx::{Database, Error as SqlxError, Pool};

#[derive(Debug)]
pub enum DatabaseError {
    NotFound(String),
    CommunicationError,
    ConnectionFailed,
    ConnectionNotAvailable,
    QueryFailed(String),
    ColumnNotFound(String),
    ProtocolNotSupported,
    NotImplemented,
    Unknown(String),
    DatabaseInconsistence(String),
    MigrationFailed(String),
}

impl From<SqlxError> for DatabaseError {
    fn from(value: SqlxError) -> Self {
        match value {
            SqlxError::ColumnNotFound(column_name) => Self::ColumnNotFound(column_name),
            SqlxError::Io(_) | SqlxError::Tls(_) => Self::CommunicationError,
            SqlxError::PoolTimedOut => Self::ConnectionNotAvailable,
            SqlxError::Database(e) => Self::QueryFailed(e.to_string()),
            SqlxError::Protocol(_) => Self::ProtocolNotSupported,
            SqlxError::TypeNotFound { type_name } => {
                Self::DatabaseInconsistence(format!("TypeNotFound {type_name}"))
            }
            _ => Self::ConnectionFailed,
        }
    }
}

#[async_trait::async_trait]
pub trait EntityRepository<
    Db: Database,
    Entity: Send,
    CreateInput: Send,
    UpdateInput: Send,
    QueryOne: Send + Sync,
    QueryMany: Send + Sync,
>
{
    async fn insert(db: &Pool<Db>, input: CreateInput) -> Result<Entity, DatabaseError>;
    async fn delete(db: &Pool<Db>, key: QueryOne) -> Result<Entity, DatabaseError>;
    async fn update(
        db: &Pool<Db>,
        key: QueryOne,
        update: UpdateInput,
    ) -> Result<Entity, DatabaseError>;
    async fn get(db: &Pool<Db>, key: QueryOne) -> Result<Entity, DatabaseError>;
    async fn try_get(db: &Pool<Db>, key: QueryOne) -> Result<Option<Entity>, DatabaseError>;
}
