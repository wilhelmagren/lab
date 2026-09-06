use arrow::array::RecordBatch;
use arrow::util::pretty::pretty_format_batches;
use bytes::Bytes;

use tqe::context::SessionContext;
use tqe::logical::expr::avg;
use tqe::logical::expr::col;
use tqe::logical::expr::max;
use tqe::logical::expr::min;

fn main() {
    let ctx = SessionContext::new();

    let data = std::fs::read("./data/weather_stations_small.parquet").unwrap();

    let df = ctx
        .parquet_bytes("weather_stations", Bytes::from(data))
        .agg(
            vec![col("station_name")],
            vec![
                min(col("measurement")),
                max(col("measurement")),
                avg(col("measurement")),
            ],
        );

    let results: Vec<RecordBatch> = ctx.execute(&df).collect();

    println!("{}", pretty_format_batches(&results).unwrap());

    /*
    println!("======= TITANIC QUERY =======");
    let df = ctx
        .parquet("./data/titanic.parquet")
        .agg(vec![col("Pclass"), col("Sex")], vec![sum(col("Survived"))]);

    let results: Vec<RecordBatch> = ctx.execute(&df).collect();
    println!("{}", pretty_format_batches(&results).unwrap());

    println!("======= 1BRC QUERY =======");
    let df = ctx
        .parquet("./data/weather_stations.parquet")
        // .filter(col("station_name").eq("Hualien"))
        .agg(
            vec![col("station_name")],
            vec![
                min(col("measurement")),
                max(col("measurement")),
                avg(col("measurement")),
            ],
        );

    let results: Vec<RecordBatch> = ctx.execute(&df).collect();
    println!("{}", pretty_format_batches(&results).unwrap());
    */
}
