pub mod array;
pub mod data_source;
pub mod record_batch;
pub mod schema;

use data_source::DataSource;

fn main() {
    let mut ds = data_source::ParquetDataSource::new("data/titanic.parquet".into());
    for rb in ds.scan(&["PassengerId", "Survived", "Pclass", "Name"]) {
        println!(
            "ColCount: {}, RowCount: {}",
            rb.column_count(),
            rb.row_count()
        );
    }
}
