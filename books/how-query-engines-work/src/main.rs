pub mod context;
pub mod data_source;
pub mod dataframe;
pub mod logical;
pub mod optimizer;
pub mod physical;
pub mod planner;
pub mod scalar;
pub mod sql;

use arrow::array::RecordBatch;
use arrow::util::pretty::pretty_format_batches;

use crate::context::SessionContext;
use crate::logical::expr::avg;
use crate::logical::expr::col;
use crate::logical::expr::max;
use crate::logical::expr::min;
use crate::logical::expr::sum;

fn main() {
    let ctx = SessionContext::new();

    /*
    println!("======= TITANIC QUERY =======");
    let df = ctx
        .parquet("./data/titanic.parquet")
        .agg(vec![col("Pclass"), col("Sex")], vec![sum(col("Survived"))]);

    let results: Vec<RecordBatch> = ctx.execute(&df).collect();
    println!("{}", pretty_format_batches(&results).unwrap());
    */

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
    // println!("{}", pretty_format_batches(&results).unwrap());
}
