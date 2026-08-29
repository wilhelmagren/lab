use crate::array::{ColumnArrayRef, new_empty_column_array};
use crate::schema::SchemaRef;

/// Record batches enable batch processing; process chunks of like 10k rows instead of row-by-row.
#[allow(dead_code)]
pub struct RecordBatch {
    schema: SchemaRef,
    columns: Vec<ColumnArrayRef>,

    row_count: usize,
}

impl RecordBatch {
    pub fn new(schema: SchemaRef, columns: Vec<ColumnArrayRef>, row_count: usize) -> Self {
        Self {
            schema,
            columns,
            row_count,
        }
    }

    pub fn new_empty(schema: SchemaRef) -> Self {
        let columns = schema
            .fields()
            .iter()
            .map(|f| new_empty_column_array(f.data_type()))
            .collect();

        Self {
            schema,
            columns,
            row_count: 0,
        }
    }

    pub fn row_count(&self) -> usize {
        self.columns[0].size()
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn field(&self, idx: usize) -> ColumnArrayRef {
        // ARc.clone() cheap
        self.columns[idx].clone()
    }
}
