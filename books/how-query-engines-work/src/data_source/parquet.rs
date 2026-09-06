use std::fs;

use arrow::datatypes::SchemaRef;
use bytes::Bytes;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::schema::types::SchemaDescriptor;
use parquet::{arrow::ProjectionMask, file::reader::ChunkReader};

use crate::data_source::{DataSource, RecordBatchIterator};

enum ParquetInput {
    #[cfg(not(target_arch = "wasm32"))]
    File(String),
    Memory(Bytes),
}

pub struct ParquetDataSource {
    name: String,
    input: ParquetInput,
    schema: SchemaRef,
    parquet_schema: SchemaDescriptor,
}

impl ParquetDataSource {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(filename: impl Into<String>) -> Self {
        let filename = filename.into();

        let builder =
            ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&filename).unwrap()).unwrap();

        Self {
            name: filename.clone(),
            input: ParquetInput::File(filename),
            schema: builder.schema().clone(),
            parquet_schema: builder.parquet_schema().clone(),
        }
    }

    pub fn from_bytes(name: impl Into<String>, bytes: Bytes) -> Self {
        let builder = ParquetRecordBatchReaderBuilder::try_new(bytes.clone()).unwrap();

        Self {
            name: name.into(),
            input: ParquetInput::Memory(bytes),
            schema: builder.schema().clone(),
            parquet_schema: builder.parquet_schema().clone(),
        }
    }
}

/// filesystem                    browser
///    │                            │
///    ▼                            ▼
///  File                         Bytes
///    │                            │
///    └────────────┬───────────────┘
///                 ▼
///      ParquetRecordBatchReader
///                 │
///                 ▼
///           RecordBatch
///                 │
///                 ▼
///       your existing engine

impl DataSource for ParquetDataSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn scan(&self, projection: Option<&[usize]>) -> RecordBatchIterator {
        match &self.input {
            #[cfg(not(target_arch = "wasm32"))]
            ParquetInput::File(filename) => {
                self.scan_reader(fs::File::open(filename).unwrap(), projection)
            }

            ParquetInput::Memory(bytes) => self.scan_reader(bytes.clone(), projection),
        }
    }
}

impl ParquetDataSource {
    fn scan_reader<T>(&self, reader: T, projection: Option<&[usize]>) -> RecordBatchIterator
    where
        T: ChunkReader + 'static,
    {
        let mut builder = ParquetRecordBatchReaderBuilder::try_new(reader).unwrap();

        if let Some(projection) = projection {
            builder = builder.with_projection(ProjectionMask::roots(
                &self.parquet_schema,
                projection.iter().copied(),
            ));
        }

        let reader = builder.build().unwrap();

        Box::new(reader.map(|batch| batch.unwrap()))
    }
}
