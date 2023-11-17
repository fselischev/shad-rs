use std::{borrow::Cow, fmt};

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ObjectId(i64);

impl ObjectId {
    pub fn into_i64(self) -> i64 {
        self.0
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

impl From<i64> for ObjectId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<ObjectId> for i64 {
    fn from(value: ObjectId) -> Self {
        value.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub enum SqlType {
    Text,
    Blob,
    BigInt,
    Real,
    TinyInt,
}

impl ToString for SqlType {
    fn to_string(&self) -> String {
        String::from(match self {
            SqlType::Text => "TEXT",
            SqlType::Blob => "BLOB",
            SqlType::BigInt => "BIGINT",
            SqlType::Real => "REAL",
            SqlType::TinyInt => "TINYINT",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    String,
    Bytes,
    Int64,
    Float64,
    Bool,
}

impl DataType {
    pub fn to_sql_type(&self) -> SqlType {
        match self {
            DataType::String => SqlType::Text,
            DataType::Bytes => SqlType::Blob,
            DataType::Int64 => SqlType::BigInt,
            DataType::Float64 => SqlType::Real,
            DataType::Bool => SqlType::TinyInt,
        }
    }
}

pub trait AsDataType {
    const DATA_TYPE: DataType;
}

macro_rules! impl_as_data_type {
    ($type:ty, $data_type:ident) => {
        impl AsDataType for $type {
            const DATA_TYPE: DataType = DataType::$data_type;
        }
    };
}

impl_as_data_type!(String, String);
impl_as_data_type!(Vec<u8>, Bytes);
impl_as_data_type!(i64, Int64);
impl_as_data_type!(f64, Float64);
impl_as_data_type!(bool, Bool);

////////////////////////////////////////////////////////////////////////////////

pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}

impl<'a> rusqlite::ToSql for Value<'a> {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match self {
            Value::String(s) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(s.clone().into_owned()),
            )),
            Value::Bytes(bytes) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Blob(bytes.clone().into_owned()),
            )),
            Value::Int64(i) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(*i),
            )),
            Value::Float64(f) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Real(*f),
            )),
            Value::Bool(b) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
            )),
        }
    }
}

pub trait IntoDataType<T> {
    fn into(self) -> T;
}

macro_rules! impl_into_datatype {
    ($result:ty, $variant:ident) => {
        impl<'a> IntoDataType<$result> for Value<'a> {
            fn into(self) -> $result {
                match self {
                    Value::$variant(inner) => inner.into(),
                    _ => panic!("not convertable into DataType"),
                }
            }
        }
    };
}

impl_into_datatype!(String, String);
impl_into_datatype!(Vec<u8>, Bytes);
impl_into_datatype!(i64, Int64);
impl_into_datatype!(f64, Float64);
impl_into_datatype!(bool, Bool);

impl<'a> From<&'a String> for Value<'a> {
    fn from(value: &'a String) -> Self {
        Value::String(value.into())
    }
}

impl<'a> From<&'a Vec<u8>> for Value<'a> {
    fn from(value: &'a Vec<u8>) -> Self {
        Value::Bytes(value.into())
    }
}

impl<'a> From<&'a i64> for Value<'static> {
    fn from(value: &'a i64) -> Self {
        Value::Int64(*value)
    }
}

impl<'a> From<&'a f64> for Value<'static> {
    fn from(value: &'a f64) -> Self {
        Value::Float64(*value)
    }
}

impl<'a> From<&'a bool> for Value<'static> {
    fn from(value: &'a bool) -> Self {
        Value::Bool(*value)
    }
}
