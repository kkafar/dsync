use diesel::result::DatabaseErrorKind;

#[derive(thiserror::Error, Debug)]
pub(crate) enum LocalServerBaseInfoError {
    #[error("No local server base info present")]
    Uninitialized,

    #[error("Invalid record count: `{0}`")]
    InvalidRecordCount(usize),

    #[error("Other error `{0}`")]
    Other(anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum SaveLocalGroupError {
    #[error("Group with id: {group_id} already exists")]
    AlreadyExists { group_id: String },

    #[error("Other error when saving local group")]
    Other,
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum FileAddError {
    #[error("File with name {file_name} is already added")]
    AlreadyExists { file_name: String },

    #[error("Database error occurred: {kind:?}")]
    OtherDatabaseError { kind: DatabaseErrorKind },

    #[error("Other error: `{0:?}`")]
    Other(anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteLocalGroupError {
    #[error("No record in db for given group")]
    DoesNotExist,

    #[error("Other database error: `{0}`")]
    Other(anyhow::Error),
}

impl From<LocalServerBaseInfoError> for tonic::Status {
    fn from(value: LocalServerBaseInfoError) -> Self {
        tonic::Status::internal(format!("{}", value))
    }
}
