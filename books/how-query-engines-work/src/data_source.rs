// use arrow_csv::infer_schema_from_files;
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::schema::types::SchemaDescriptor;

use std::fs;
use std::sync::Arc;

use crate::array::column_arrays_from_arrow;
use crate::record_batch::RecordBatch;
use crate::schema::{Schema, SchemaRef};

pub struct ScanProjection {
    names: Vec<String>,
}

impl ScanProjection {
    pub fn new(names: Vec<String>) -> Self {
        Self { names }
    }

    pub fn indices(&self, schema: SchemaRef) -> Vec<usize> {
        self.names
            .iter()
            .map(|n| schema.fields().find(n).unwrap().0)
            .collect()
    }
}

impl std::fmt::Display for ScanProjection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.names.join(","))
    }
}

pub type DataSourceRef = Arc<dyn DataSource>;
pub type RecordBatchIteratorRef<'a> = Box<dyn Iterator<Item = RecordBatch> + 'a>;

pub trait DataSource {
    fn name(&self) -> &str;
    fn schema(&self) -> SchemaRef;
    fn scan(&self, projection: Option<ScanProjection>) -> RecordBatchIteratorRef<'_>;
}

/*
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
    fn name(&self) -> &str {
        &self.filename
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn scan(&self, projection: Option<ScanProjection>) -> RecordBatchIteratorRef<'_> {
        let mut builder =
            ParquetRecordBatchReaderBuilder::try_new(fs::File::open(&self.filename).unwrap())
                .unwrap();

        if let Some(proj) = projection {
            builder = builder.with_projection(ProjectionMask::roots(
                &self.parquet_schema,
                proj.indices(self.schema.clone()),
            ));

            // we need to update the data source schema with only projected cols
            // self.schema = Arc::new(self.schema.project(proj.indices(self.schema.clone())));
        };

        let reader = builder.build().unwrap();

        let iter = reader
            .into_iter()
            // fuck it we ball, what could go wrong
            .map(|maybe_rb| maybe_rb.unwrap())
            .map(|rb| {
                RecordBatch::new(
                    self.schema.clone(),
                    column_arrays_from_arrow(rb.columns()),
                    rb.num_rows(),
                )
            });

        Box::new(iter)
    }
}
