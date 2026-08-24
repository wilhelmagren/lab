use arrow::{
    array::{
        Array, ArrayRef, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array,
        Int32Array, Int64Array, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    },
    datatypes::{DataType, SchemaRef as ArrowSchemaRef},
};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
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

    pub fn dtype(&self) -> &DataType {
        &self.dtype
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Schema {
    fields: Vec<Field>,
}

impl Schema {
    pub fn new(fields: impl Iterator<Item = Field>) -> Self {
        Self {
            fields: fields.collect(),
        }
    }

    pub fn from_arrow(schema: ArrowSchemaRef) -> Self {
        Self {
            fields: schema
                .fields()
                .iter()
                .map(|f| Field::new(f.name(), f.data_type().clone(), f.is_nullable()))
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

#[derive(Clone, Debug, PartialEq)]
enum ScalarValue {
    Boolean(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Utf8(String),
    Null,
}

trait ColumnArray {
    fn dtype(&self) -> &DataType;
    fn get_value(&self, idx: usize) -> ScalarValue;
    fn size(&self) -> usize;
}

struct FieldArray {
    dtype: DataType,
    inner: ArrayRef,
}

impl ColumnArray for FieldArray {
    fn dtype(&self) -> &DataType {
        &self.dtype
    }

    fn get_value(&self, idx: usize) -> ScalarValue {
        if self.inner.is_null(idx) {
            return ScalarValue::Null;
        }

        match &self.dtype {
            DataType::Boolean => {
                let arr = self.inner.as_any().downcast_ref::<BooleanArray>().unwrap();
                return ScalarValue::Boolean(arr.value(idx));
            }
            DataType::Int8 => {
                let arr = self.inner.as_any().downcast_ref::<Int8Array>().unwrap();
                return ScalarValue::Int8(arr.value(idx));
            }
            DataType::Int16 => {
                let arr = self.inner.as_any().downcast_ref::<Int16Array>().unwrap();
                return ScalarValue::Int16(arr.value(idx));
            }
            DataType::Int32 => {
                let arr = self.inner.as_any().downcast_ref::<Int32Array>().unwrap();
                return ScalarValue::Int32(arr.value(idx));
            }
            DataType::Int64 => {
                let arr = self.inner.as_any().downcast_ref::<Int64Array>().unwrap();
                return ScalarValue::Int64(arr.value(idx));
            }
            DataType::UInt8 => {
                let arr = self.inner.as_any().downcast_ref::<UInt8Array>().unwrap();
                return ScalarValue::UInt8(arr.value(idx));
            }
            DataType::UInt16 => {
                let arr = self.inner.as_any().downcast_ref::<UInt16Array>().unwrap();
                return ScalarValue::UInt16(arr.value(idx));
            }
            DataType::UInt32 => {
                let arr = self.inner.as_any().downcast_ref::<UInt32Array>().unwrap();
                return ScalarValue::UInt32(arr.value(idx));
            }
            DataType::UInt64 => {
                let arr = self.inner.as_any().downcast_ref::<UInt64Array>().unwrap();
                return ScalarValue::UInt64(arr.value(idx));
            }
            DataType::Float32 => {
                let arr = self.inner.as_any().downcast_ref::<Float32Array>().unwrap();
                return ScalarValue::Float32(arr.value(idx));
            }
            DataType::Float64 => {
                let arr = self.inner.as_any().downcast_ref::<Float64Array>().unwrap();
                return ScalarValue::Float64(arr.value(idx));
            }
            DataType::Utf8 => {
                let arr = self.inner.as_any().downcast_ref::<StringArray>().unwrap();
                return ScalarValue::Utf8(arr.value(idx).to_string());
            }
            d => panic!("Unsupported datatype {:?}", d),
        }
    }

    fn size(&self) -> usize {
        self.inner.len()
    }
}

/// this is a virtual column
struct LiteralValueArray {
    dtype: DataType,
    value: ScalarValue,
    size: usize,
}

impl ColumnArray for LiteralValueArray {
    fn dtype(&self) -> &DataType {
        &self.dtype
    }

    fn get_value(&self, idx: usize) -> ScalarValue {
        if idx >= self.size {
            panic!("index out of bounds yoo")
        }

        self.value.clone()
    }

    fn size(&self) -> usize {
        self.size
    }
}

type ColumnArrayRef = Arc<dyn ColumnArray>;
type SchemaRef = Arc<Schema>;

struct RecordBatch {
    schema: Schema,
    columns: Vec<ColumnArrayRef>,
}

impl RecordBatch {
    fn row_count(&self) -> usize {
        self.columns[0].size()
    }

    fn column_count(&self) -> usize {
        self.columns.len()
    }

    fn field(&self, idx: usize) -> ColumnArrayRef {
        self.columns[idx].clone()
    }
}

trait DataSource {
    fn schema(&self) -> SchemaRef;
    fn scan(&self, projection: impl Iterator<Item = String>) -> impl Iterator<Item = RecordBatch>;
}

fn main() {
    println!("Hello, world!");
}
