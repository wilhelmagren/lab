pub mod array;
pub mod data_source;
pub mod logical_plan;
pub mod record_batch;
pub mod schema;

use std::time::Instant;

use data_source::DataSource;

// 1 billion rows:
// my record batch takes 15seconds
// arrow record batches takes same amount of time :)
fn main() {
    let now = Instant::now();
    let mut ds = data_source::ParquetDataSource::new("data/weather_stations_small.parquet".into());

    let mut n_rows = 0;
    let mut n_batches = 0;

    for rb in ds.scan(&["station_name", "measurement"]).into_iter() {
        n_rows += rb.row_count();
        n_batches += 1;
    }

    let elapsed = now.elapsed();

    println!(
        "{n_rows} rows across {n_batches} batches, took {}s ({}ms, {}μs)",
        elapsed.as_secs(),
        elapsed.as_millis(),
        elapsed.as_micros()
    );
}
