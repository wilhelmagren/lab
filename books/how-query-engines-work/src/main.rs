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
use crate::logical::expr::col;
use crate::logical::expr::sum;

fn main() {
    let ctx = SessionContext::new();

    let passengers = ctx.parquet("data/titanic.parquet");

    let df = passengers.agg(vec![col("Sex")], vec![sum(col("Survived"))]);

    let results: Vec<RecordBatch> = ctx.execute(&df).collect();
    println!("{}", pretty_format_batches(&results).unwrap());
}
