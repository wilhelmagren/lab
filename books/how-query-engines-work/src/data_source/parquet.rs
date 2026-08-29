use std::fs;

use arrow::datatypes::SchemaRef;
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::schema::types::SchemaDescriptor;

use crate::data_source::{DataSource, RecordBatchIterator};

pub struct ParquetDataSource {
    filename: String,
    schema: SchemaRef,
    parquet_schema: SchemaDescriptor,
}

impl ParquetDataSource {
    pub fn new(filename: impl Into<String>) -> Self {
        let filename = filename.into();

        let builder =
            ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&filename).unwrap()).unwrap();

        Self {
            filename,
            schema: builder.schema().clone(),
            parquet_schema: builder.parquet_schema().clone(),
        }
    }
}

impl DataSource for ParquetDataSource {
    fn name(&self) -> &str {
        &self.filename
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn scan(&self, projection: Option<&[usize]>) -> RecordBatchIterator<'_> {
        let mut builder =
            ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&self.filename).unwrap())
                .unwrap();

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
