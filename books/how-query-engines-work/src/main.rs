pub mod context;
pub mod data_source;
pub mod dataframe;
pub mod logical;
pub mod optimizer;
pub mod physical;
pub mod scalar;

use context::SessionContext;

use crate::logical::expr::col;

fn main() {
    SessionContext::new()
        .parquet("data/titanic.parquet")
        .filter(col("Age").gteq(18) & col("Sex").eq("female") & col("Fare").gt(20.0))
        .project(vec![
            col("Name"),
            col("Age"),
            col("Pclass"),
            col("Fare"),
            (col("Fare") / col("Pclass")).alias("fare_per_class"),
        ])
        .filter(col("fare_per_class").gt(15.0))
        .limit(10)
        .print_plan();
}
