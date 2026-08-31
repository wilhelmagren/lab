pub mod parquet;
pub mod registry;

use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;

pub type RecordBatchIterator = Box<dyn Iterator<Item = RecordBatch>>;
pub type DataSourceRef = Arc<dyn DataSource>;

pub trait DataSource {
    fn name(&self) -> &str;
    fn schema(&self) -> &SchemaRef;
    fn scan(&self, projection: Option<&[usize]>) -> RecordBatchIterator;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(u64);

impl SourceId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}
