// TODO: type coercion

use arrow::{
    array::{
        Array, ArrayRef, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array,
        Int32Array, Int64Array, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
        new_empty_array,
    },
    datatypes::DataType,
};
use std::sync::Arc;

pub type ColumnArrayRef = Arc<dyn ColumnArray>;

pub trait ColumnArray: std::fmt::Debug {
    fn dtype(&self) -> &DataType;
    fn get_value(&self, idx: usize) -> ScalarValue;
    fn size(&self) -> usize;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScalarValue {
    Null,

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
}

impl ScalarValue {
    pub fn dtype(&self) -> DataType {
        match self {
            ScalarValue::Null => DataType::Null,
            ScalarValue::Boolean(_) => DataType::Boolean,
            ScalarValue::Int8(_) => DataType::Int8,
            ScalarValue::Int16(_) => DataType::Int16,
            ScalarValue::Int32(_) => DataType::Int32,
            ScalarValue::Int64(_) => DataType::Int64,
            ScalarValue::UInt8(_) => DataType::UInt8,
            ScalarValue::UInt16(_) => DataType::UInt16,
            ScalarValue::UInt32(_) => DataType::UInt32,
            ScalarValue::UInt64(_) => DataType::UInt64,
            ScalarValue::Float32(_) => DataType::Float32,
            ScalarValue::Float64(_) => DataType::Float64,
            ScalarValue::Utf8(_) => DataType::Utf8,
        }
    }
}

impl std::fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarValue::Null => write!(f, "NULL"),
            ScalarValue::Boolean(v) => write!(f, "{}", v),
            ScalarValue::Int8(v) => write!(f, "{}", v),
            ScalarValue::Int16(v) => write!(f, "{}", v),
            ScalarValue::Int32(v) => write!(f, "{}", v),
            ScalarValue::Int64(v) => write!(f, "{}", v),
            ScalarValue::UInt8(v) => write!(f, "{}", v),
            ScalarValue::UInt16(v) => write!(f, "{}", v),
            ScalarValue::UInt32(v) => write!(f, "{}", v),
            ScalarValue::UInt64(v) => write!(f, "{}", v),
            ScalarValue::Float32(v) => write!(f, "{}", v),
            ScalarValue::Float64(v) => write!(f, "{}", v),
            ScalarValue::Utf8(v) => write!(f, "'{}'", v),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FieldArray {
    dtype: DataType,
    inner: ArrayRef,
}

impl FieldArray {
    pub fn new(dtype: DataType, inner: ArrayRef) -> Self {
        Self { dtype, inner }
    }
}

impl ColumnArray for FieldArray {
    fn dtype(&self) -> &DataType {
        &self.dtype
    }

    // todo: make this use value_unchecked so we get faster lookup :) UNSAFE UH OH
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
            d => panic!("unsupported datatype {:?}", d),
        }
    }

    fn size(&self) -> usize {
        self.inner.len()
    }
}

/// this is a virtual column
#[derive(Clone, Debug)]
pub struct LiteralValueArray {
    dtype: DataType,
    value: ScalarValue,
    size: usize,
}

impl LiteralValueArray {
    pub fn new(dtype: DataType, value: ScalarValue, size: usize) -> Self {
        Self { dtype, value, size }
    }
}

impl ColumnArray for LiteralValueArray {
    fn dtype(&self) -> &DataType {
        &self.dtype
    }

    fn get_value(&self, idx: usize) -> ScalarValue {
        if idx >= self.size {
            panic!("index out of bounds yoo")
        }

        // cheap if self is not of ScalarValue::String dtype
        self.value.clone()
    }

    fn size(&self) -> usize {
        self.size
    }
}

/// Create a new empty column array with the associated datatype.
pub fn new_empty_column_array(dtype: &DataType) -> ColumnArrayRef {
    Arc::new(FieldArray::new(dtype.clone(), new_empty_array(dtype)))
}

pub fn column_arrays_from_arrow(columns: &[ArrayRef]) -> Vec<ColumnArrayRef> {
    columns
        .iter()
        .map(|arr| {
            Arc::new(FieldArray::new(arr.data_type().clone(), arr.clone())) as ColumnArrayRef
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use arrow::array::Int32Builder;

    use super::*;

    #[test]
    fn field_array_trait_impls() {
        let mut b = Int32Builder::new();
        b.append_nulls(2);
        b.append_value_n(1337, 4);

        let fa = FieldArray::new(DataType::Int32, Arc::new(b.finish()));
        assert_eq!(&DataType::Int32, fa.dtype());
        assert_eq!(ScalarValue::Null, fa.get_value(0));
        assert_eq!(ScalarValue::Int32(1337), fa.get_value(4));
        assert_eq!(6, fa.size());
    }

    #[test]
    fn literal_value_array_trait_impls() {
        let lva =
            LiteralValueArray::new(DataType::Float32, ScalarValue::Float32(-124.12309), 100000);
        assert_eq!(&DataType::Float32, lva.dtype());
        assert_eq!(ScalarValue::Float32(-124.12309), lva.get_value(99999));
    }

    #[test]
    #[should_panic]
    fn literal_value_array_trait_impls_indx_oob() {
        let lva = LiteralValueArray::new(DataType::Float32, ScalarValue::Float32(-124.12309), 1337);
        assert_eq!(ScalarValue::Float32(-124.12309), lva.get_value(2001));
    }
}
