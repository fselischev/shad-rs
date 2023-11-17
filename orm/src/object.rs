use crate::{data::DataType, storage::Row};

use std::any::Any;

////////////////////////////////////////////////////////////////////////////////

pub trait Object: Any {
    fn as_table_row(&self) -> Row;
    fn from_table_row(row: Row) -> Self;
    fn schema() -> &'static Schema;
}

////////////////////////////////////////////////////////////////////////////////

pub struct Schema {
    pub type_name: &'static str,
    pub table_name: &'static str,
    pub attrs: &'static [Attribute],
}

impl Schema {
    pub fn find_attr_by_col(&self, col_name: &str) -> Option<&Attribute> {
        self.attrs.iter().find(|&attr| attr.col_name == col_name)
    }
}

pub struct Attribute {
    pub name: &'static str,
    pub col_name: &'static str,
    pub data_type: DataType,
}

pub trait Store {
    fn as_table_row(&self) -> Row;
    fn schema(&self) -> &'static Schema;
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Object> Store for T {
    fn as_table_row(&self) -> Row {
        Object::as_table_row(self)
    }

    fn schema(&self) -> &'static Schema {
        Self::schema()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
