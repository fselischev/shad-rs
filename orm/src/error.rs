use crate::{data::DataType, object::Schema, ObjectId};

use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFound(Box<NotFoundError>),
    #[error(transparent)]
    UnexpectedType(Box<UnexpectedTypeError>),
    #[error(transparent)]
    MissingColumn(Box<MissingColumnError>),
    #[error("database is locked")]
    LockConflict,
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error>),
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(_, _) => Self::LockConflict,
            err => Self::Storage(Box::new(err)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("object is not found: type '{type_name}', id {object_id}")]
pub struct NotFoundError {
    pub object_id: ObjectId,
    pub type_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "invalid type for {type_name}::{attr_name}: expected equivalent of {expected_type:?}, \
    got {got_type} (table: {table_name}, column: {column_name})"
)]
pub struct UnexpectedTypeError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
    pub expected_type: DataType,
    pub got_type: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "missing a column for {type_name}::{attr_name} \
    (table: {table_name}, column: {column_name})"
)]
pub struct MissingColumnError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
}

const MISSING_COLUMN_PREF_FIRST: &str = "no such column: ";
const MISSING_COLUMN_PREF_SECOND: &str = "has no column named";
const MISSING_COLUMN_PREFS: &[&str] = &[MISSING_COLUMN_PREF_FIRST, MISSING_COLUMN_PREF_SECOND];

impl MissingColumnError {
    fn try_from_msg(msg: &str, schema: &Schema) -> Option<Self> {
        for pref in MISSING_COLUMN_PREFS {
            let pos = match msg.find(pref) {
                Some(pos) => pos,
                None => continue,
            };

            let attr = schema
                .find_attr_by_col(msg[pos + pref.len()..].trim())
                .unwrap();
            return Some(Self {
                type_name: schema.type_name,
                attr_name: attr.name,
                table_name: schema.table_name,
                column_name: attr.col_name,
            });
        }

        None
    }
}

////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, Error>;

pub trait MapErr<T> {
    fn map_col_err(self, schema: &Schema) -> Result<T>;
    fn map_table_err(self, schema: &Schema, id: ObjectId) -> Result<T>;
}

impl<T> MapErr<T> for std::result::Result<T, rusqlite::Error> {
    fn map_col_err(self, schema: &Schema) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => match err {
                rusqlite::Error::SqliteFailure(_, msg) => Err(Error::MissingColumn(Box::new(
                    MissingColumnError::try_from_msg(&msg.unwrap(), schema).unwrap(),
                ))),
                rusqlite::Error::InvalidColumnType(n, _, ty) => {
                    Err(Error::UnexpectedType(Box::new(UnexpectedTypeError {
                        type_name: schema.type_name,
                        attr_name: schema.attrs[n].name,
                        table_name: schema.table_name,
                        column_name: schema.attrs[n].col_name,
                        expected_type: schema.attrs[n].data_type,
                        got_type: ty.to_string(),
                    })))
                }
                _ => panic!("Unknown sqlite error"),
            },
        }
    }

    fn map_table_err(self, schema: &Schema, id: ObjectId) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => match err {
                rusqlite::Error::SqliteFailure(_, msg) => Err(Error::MissingColumn(Box::new(
                    MissingColumnError::try_from_msg(&msg.unwrap(), schema).unwrap(),
                ))),
                rusqlite::Error::InvalidColumnType(n, _, ty) => {
                    Err(Error::UnexpectedType(Box::new(UnexpectedTypeError {
                        type_name: schema.type_name,
                        attr_name: schema.attrs[n].name,
                        table_name: schema.table_name,
                        column_name: schema.attrs[n].col_name,
                        expected_type: schema.attrs[n].data_type,
                        got_type: ty.to_string(),
                    })))
                }
                rusqlite::Error::QueryReturnedNoRows => {
                    Err(Error::NotFound(Box::new(NotFoundError {
                        object_id: id,
                        type_name: schema.type_name,
                    })))
                }
                _ => panic!("Unknown sqlite error"),
            },
        }
    }
}
