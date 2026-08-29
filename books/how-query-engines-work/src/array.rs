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
            Self::Null => DataType::Null,
            Self::Boolean(_) => DataType::Boolean,
            Self::Int8(_) => DataType::Int8,
            Self::Int16(_) => DataType::Int16,
            Self::Int32(_) => DataType::Int32,
            Self::Int64(_) => DataType::Int64,
            Self::UInt8(_) => DataType::UInt8,
            Self::UInt16(_) => DataType::UInt16,
            Self::UInt32(_) => DataType::UInt32,
            Self::UInt64(_) => DataType::UInt64,
            Self::Float32(_) => DataType::Float32,
            Self::Float64(_) => DataType::Float64,
            Self::Utf8(_) => DataType::Utf8,
        }
    }
}

impl std::fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Int8(v) => write!(f, "{}", v),
            Self::Int16(v) => write!(f, "{}", v),
            Self::Int32(v) => write!(f, "{}", v),
            Self::Int64(v) => write!(f, "{}", v),
            Self::UInt8(v) => write!(f, "{}", v),
            Self::UInt16(v) => write!(f, "{}", v),
            Self::UInt32(v) => write!(f, "{}", v),
            Self::UInt64(v) => write!(f, "{}", v),
            Self::Float32(v) => write!(f, "{}", v),
            Self::Float64(v) => write!(f, "{}", v),
            Self::Utf8(v) => write!(f, "'{}'", v),
        }
    }
}

// this is a hack to make us easy do lit(None), but doesn't work for lit(Some...) obvsiously
impl From<Option<bool>> for ScalarValue {
    fn from(_value: Option<bool>) -> Self {
        Self::Null
    }
}

impl From<bool> for ScalarValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i8> for ScalarValue {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<i16> for ScalarValue {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<i32> for ScalarValue {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<i64> for ScalarValue {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<u8> for ScalarValue {
    fn from(value: u8) -> Self {
        Self::UInt8(value)
    }
}

impl From<u16> for ScalarValue {
    fn from(value: u16) -> Self {
        Self::UInt16(value)
    }
}

impl From<u32> for ScalarValue {
    fn from(value: u32) -> Self {
        Self::UInt32(value)
    }
}

impl From<u64> for ScalarValue {
    fn from(value: u64) -> Self {
        Self::UInt64(value)
    }
}

impl From<f32> for ScalarValue {
    fn from(value: f32) -> Self {
        Self::Float32(value)
    }
}

impl From<f64> for ScalarValue {
    fn from(value: f64) -> Self {
        Self::Float64(value)
    }
}

impl From<&str> for ScalarValue {
    fn from(value: &str) -> Self {
        Self::Utf8(value.to_string())
    }
}

impl From<String> for ScalarValue {
    fn from(value: String) -> Self {
        Self::Utf8(value)
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
