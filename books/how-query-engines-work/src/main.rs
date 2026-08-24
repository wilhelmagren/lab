use arrow::{
    array::{
        Array, ArrayData, ArrayRef, BooleanArray, Float32Array, Float64Array, Int8Array,
        Int16Array, Int32Array, Int64Array, StringArray, UInt8Array, UInt16Array, UInt32Array,
        UInt64Array,
    },
    datatypes::{DataType as ArrowDataType, SchemaRef},
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq)]
enum DataType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    String,
}

impl DataType {
    pub fn from_arrow(arrow_type: &ArrowDataType) -> Self {
        match arrow_type {
            ArrowDataType::Boolean => Self::Boolean,
            ArrowDataType::Int8 => Self::Int8,
            ArrowDataType::Int16 => Self::Int16,
            ArrowDataType::Int32 => Self::Int32,
            ArrowDataType::Int64 => Self::Int64,
            ArrowDataType::UInt8 => Self::UInt8,
            ArrowDataType::UInt16 => Self::UInt16,
            ArrowDataType::UInt32 => Self::UInt32,
            ArrowDataType::UInt64 => Self::UInt64,
            ArrowDataType::Float32 => Self::Float,
            ArrowDataType::Float64 => Self::Double,
            ArrowDataType::Utf8 => Self::String,
            _ => panic!("not supported datatype!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Field {
    name: String,
    dtype: DataType,
    nullable: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, dtype: DataType, nullable: bool) -> Self {
        Self {
            name: name.into(),
            dtype,
            nullable,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Schema {
    fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: impl Iterator<Item = Field>) -> Self {
        Self {
            fields: fields.collect(),
        }
    }

    pub fn from_arrow(schema: SchemaRef) -> Self {
        Self {
            fields: schema
                .fields()
                .iter()
                .map(|f| {
                    Field::new(
                        f.name(),
                        DataType::from_arrow(f.data_type()),
                        f.is_nullable(),
                    )
                })
                .collect(),
        }
    }

    pub fn project(&self, indices: impl Iterator<Item = usize>) -> Self {
        Self {
            fields: indices.map(|i| self.fields[i].clone()).collect(),
        }
    }

    pub fn select(&self, names: impl Iterator<Item = String>) -> Self {
        let mut selected = Vec::new();

        for name in names {
            let matches: Vec<&Field> = self.fields.iter().filter(|f| f.name == name).collect();
            if matches.len() == 1 {
                selected.push(matches[0].clone());
            } else {
                panic!("UH oh stinky")
            }
        }

        Self { fields: selected }
    }
}

fn main() {
    println!("Hello, world!");
}
