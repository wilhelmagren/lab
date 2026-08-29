use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array,
    Int64Array, NullArray, Scalar, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::DataType;

fn _make_scalar<A>(s: Scalar<A>) -> Scalar<ArrayRef>
where
    A: Array + 'static,
{
    Scalar::new(Arc::new(s.into_inner()) as ArrayRef)
}

#[derive(Clone)]
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
    // i like calling it dtype but in arrow the methods are called data_type so we use the same...
    pub fn data_type(&self) -> DataType {
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

    pub fn to_arrow_scalar(&self) -> Scalar<ArrayRef> {
        match self {
            Self::Null => Scalar::new(Arc::new(NullArray::new(1)) as ArrayRef),
            Self::Boolean(v) => _make_scalar(BooleanArray::new_scalar(*v)),
            Self::Int8(v) => _make_scalar(Int8Array::new_scalar(*v)),
            Self::Int16(v) => _make_scalar(Int16Array::new_scalar(*v)),
            Self::Int32(v) => _make_scalar(Int32Array::new_scalar(*v)),
            Self::Int64(v) => _make_scalar(Int64Array::new_scalar(*v)),
            Self::UInt8(v) => _make_scalar(UInt8Array::new_scalar(*v)),
            Self::UInt16(v) => _make_scalar(UInt16Array::new_scalar(*v)),
            Self::UInt32(v) => _make_scalar(UInt32Array::new_scalar(*v)),
            Self::UInt64(v) => _make_scalar(UInt64Array::new_scalar(*v)),
            Self::Float32(v) => _make_scalar(Float32Array::new_scalar(*v)),
            Self::Float64(v) => _make_scalar(Float64Array::new_scalar(*v)),
            Self::Utf8(v) => _make_scalar(StringArray::new_scalar(v.as_str())),
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

macro_rules! impl_scalar_from {
    ($ty:ty, $variant:ident) => {
        impl From<$ty> for ScalarValue {
            fn from(value: $ty) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl<T> From<Option<T>> for ScalarValue
where
    T: Into<ScalarValue>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => v.into(),
            None => Self::Null,
        }
    }
}

impl From<String> for ScalarValue {
    fn from(value: String) -> Self {
        Self::Utf8(value)
    }
}

impl From<&str> for ScalarValue {
    fn from(value: &str) -> Self {
        Self::Utf8(value.to_owned())
    }
}

impl_scalar_from!(bool, Boolean);
impl_scalar_from!(i8, Int8);
impl_scalar_from!(i16, Int16);
impl_scalar_from!(i32, Int32);
impl_scalar_from!(i64, Int64);
impl_scalar_from!(u8, UInt8);
impl_scalar_from!(u16, UInt16);
impl_scalar_from!(u32, UInt32);
impl_scalar_from!(u64, UInt64);
impl_scalar_from!(f32, Float32);
impl_scalar_from!(f64, Float64);
