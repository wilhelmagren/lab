use arrow_csv::infer_schema_from_files;
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};
use parquet::schema::types::SchemaDescriptor;

use std::fs;
use std::sync::Arc;

use crate::array::column_arrays_from_arrow;
use crate::record_batch::RecordBatch;
use crate::schema::{Schema, SchemaRef};

pub trait DataSource {
    fn schema(&self) -> SchemaRef;
    fn scan(&mut self, projection: &[&str]) -> impl Iterator<Item = RecordBatch>;
}

pub struct CsvDataSource {
    filename: String,
    schema: SchemaRef,
    batch_size: usize,
}

impl CsvDataSource {
    // yo dawg I heard you like Arc's
    fn infer_schema(filename: &str) -> SchemaRef {
        Arc::new(Schema::from_arrow(Arc::new(
            infer_schema_from_files(&[filename.to_string()], 44, Some(128), true)
                .expect("could not infer schema"),
        )))
    }

    pub fn new(filename: String, schema: Option<SchemaRef>, batch_size: usize) -> Self {
        let schema = match schema {
            Some(s) => s,
            None => CsvDataSource::infer_schema(&filename),
        };

        Self {
            filename,
            schema,
            batch_size,
        }
    }
}

/*
impl DataSource for CsvDataSource {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn scan(
        &self,
        projection: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> impl Iterator<Item = RecordBatch> {
        // here we could have used arrow_csv to read the file but lets make our own batches,
        // it is good recreational programming practice :)

        // let mut reader = ReaderBuilder::new(read_schema.to_arrow())
        // .with_header(true)
        // .build(fs::File::open(&self.filename).expect("could not open csv file"))
        // .unwrap();

        // we need to get batch_size rows and parse them into 1 record batch
        // then we need to make an iterator of them, how to do all this lazy?

        todo!()
    }
}
*/

pub struct ParquetDataSource {
    filename: String,
    schema: SchemaRef,
    parquet_schema: SchemaDescriptor,
}

impl ParquetDataSource {
    pub fn new(filename: String) -> Self {
        let builder =
            ParquetRecordBatchReaderBuilder::try_new(fs::File::open(filename.clone()).unwrap())
                .unwrap();
        let schema = Arc::new(Schema::from_arrow(builder.schema().clone()));
        let parquet_schema = builder.parquet_schema();

        Self {
            filename,
            schema,
            parquet_schema: parquet_schema.clone(),
        }
    }
}

impl DataSource for ParquetDataSource {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn scan(&mut self, projection: &[&str]) -> impl Iterator<Item = RecordBatch> {
        let reader =
            ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&self.filename).unwrap())
                .unwrap()
                // fuck this shitty as api
                .with_projection(ProjectionMask::roots(
                    &self.parquet_schema,
                    self.schema.projection_mask(projection),
                ))
                .build()
                .unwrap();

        self.schema = Arc::new(self.schema.select(projection));

        reader
            .into_iter()
            // fuck it we ball, what could go wrong
            .map(|maybe_rb| maybe_rb.unwrap())
            .map(|rb| {
                RecordBatch::new(
                    self.schema.clone(),
                    column_arrays_from_arrow(rb.columns()),
                    rb.num_rows(),
                )
            })
    }
}
