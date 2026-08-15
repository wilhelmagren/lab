use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::functions_aggregate::expr_fn::{avg, max, min};
use datafusion::prelude::*;
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    let ctx = SessionContext::new(); 

    let station_field = Field::new("station", DataType::Utf8, false);
    let temp_field = Field::new("temperature", DataType::Float32, false);

    let schema = Schema::new(vec![station_field, temp_field]);

    let path = "./weather_stations.csv";

    let opts = CsvReadOptions::new()
        .delimiter(b';')
        .has_header(false)
        .file_extension("csv")
        .schema(&schema);

    let df = rt.block_on(ctx.read_csv(path, opts)).unwrap();

    let results_fut = df
        .aggregate(
            vec![col("station")],
            vec![
                min(col("temperature")).alias("min_temp"),
                avg(col("temperature")).alias("avg_temp"),
                max(col("temperature")).alias("max_temp"),
            ],
        )
        .unwrap()
        .sort(vec![col("station").sort(true, false)])
        .unwrap()
        .collect();

    let results = rt.block_on(results_fut);
    println!("{}", pretty_format_batches(&results.unwrap()).unwrap());
}
