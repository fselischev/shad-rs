use crate::{
    data::{DataType, Value},
    error::{MapErr, Result},
    object::Schema,
    ObjectId,
};

use rusqlite::OptionalExtension;

use std::{borrow::Cow, fmt::Write};

////////////////////////////////////////////////////////////////////////////////

pub type Row<'a> = Vec<Value<'a>>;
pub type RowSlice<'a> = [Value<'a>];

////////////////////////////////////////////////////////////////////////////////

pub(crate) trait StorageTransaction {
    fn table_exists(&self, table: &str) -> Result<bool>;
    fn create_table(&self, schema: &Schema) -> Result<()>;

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId>;
    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()>;
    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>>;
    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()>;

    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}

impl<'a> StorageTransaction for rusqlite::Transaction<'a> {
    fn table_exists(&self, table: &str) -> Result<bool> {
        Ok(self
            .prepare("SELECT 1 FROM sqlite_master WHERE name = ?")?
            .query_row([table], |_| Ok(()))
            .optional()?
            .is_some())
    }

    fn create_table(&self, schema: &Schema) -> Result<()> {
        let mut query = format!(
            "CREATE TABLE \"{}\" (id INTEGER PRIMARY KEY AUTOINCREMENT",
            schema.table_name
        );

        schema.attrs.iter().for_each(|attr| {
            write!(
                query,
                ", {} {}",
                attr.col_name,
                attr.data_type.to_sql_type().to_string()
            )
            .unwrap();
        });

        self.execute(&format!("{query})"), [])?;
        Ok(())
    }

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId> {
        if row.is_empty() {
            self.execute(
                &format!("INSERT INTO {} DEFAULT VALUES", schema.table_name),
                [],
            )
            .map_col_err(schema)?;
        } else {
            let query = format!(
                "INSERT INTO {}({}) VALUES({})",
                schema.table_name,
                schema
                    .attrs
                    .iter()
                    .map(|a| a.col_name)
                    .collect::<Vec<_>>()
                    .join(","),
                std::iter::repeat("?")
                    .take(row.len())
                    .collect::<Vec<_>>()
                    .join(","),
            );

            let params = row
                .iter()
                .map(|v| v as &dyn rusqlite::ToSql)
                .collect::<Vec<_>>();
            self.execute(&query, &params as &[_]).map_col_err(schema)?;
        }

        Ok(self.last_insert_rowid().into())
    }

    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()> {
        let mut query = format!(
            "UPDATE {} SET {} = ?",
            schema.table_name, schema.attrs[0].col_name
        );

        schema.attrs.iter().skip(1).for_each(|attr| {
            write!(query, ", {} = ?", attr.col_name).unwrap();
        });
        query.push_str("WHERE id = ?");

        let mut params = row
            .iter()
            .map(|v| v as &dyn rusqlite::ToSql)
            .collect::<Vec<_>>();
        params.push(id.as_i64());

        self.execute(&query, &params as &[_])
            .map_table_err(schema, id)?;
        Ok(())
    }

    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>> {
        let mut query = "SELECT ".to_string();

        if let Some(attr) = schema.attrs.first() {
            write!(query, "{}", attr.col_name).unwrap();
            schema.attrs.iter().skip(1).for_each(|attr| {
                write!(query, ", {}", attr.col_name).unwrap();
            });
        } else {
            query.push('1');
        }

        write!(query, " FROM \"{}\" WHERE id = ?", schema.table_name).unwrap();

        (move || {
            self.prepare(&query)?
                .query_row([i64::from(id)], |sqlite_row| {
                    let mut row = Row::with_capacity(schema.attrs.len());
                    for (i, attr) in schema.attrs.iter().enumerate() {
                        row.push(match attr.data_type {
                            DataType::String => Value::String(Cow::Owned(sqlite_row.get(i)?)),
                            DataType::Bytes => Value::Bytes(Cow::Owned(sqlite_row.get(i)?)),
                            DataType::Int64 => Value::Int64(sqlite_row.get(i)?),
                            DataType::Float64 => Value::Float64(sqlite_row.get(i)?),
                            DataType::Bool => Value::Bool(sqlite_row.get::<_, i64>(i)? > 0),
                        });
                    }
                    Ok(row)
                })
        })()
        .map_table_err(schema, id)
    }

    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()> {
        self.execute(
            &format!("DELETE FROM {} WHERE id = ?", schema.table_name),
            [i64::from(id)],
        )
        .map_table_err(schema, id)?;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        self.execute("COMMIT", [])?;
        Ok(())
    }

    fn rollback(&self) -> Result<()> {
        self.execute("ROLLBACK", [])?;
        Ok(())
    }
}
