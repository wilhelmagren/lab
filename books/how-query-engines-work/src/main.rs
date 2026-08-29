pub mod array;
pub mod data_source;
pub mod dataframe;
pub mod logical_plan;
pub mod record_batch;
pub mod schema;
pub mod scalar;

/*
use std::sync::Arc;

use crate::{
    data_source::{ParquetDataSource, ScanProjection},
    logical_plan::{
        Aggregate, Filter, LogicalPlan, Projection, Scan, alias, avg, column, eq, lit_double,
        lit_string, max, min, multiply,
    },
};

fn main() {
    let path = "data/weather_stations_small.parquet".to_string();
    let data_source = Arc::new(ParquetDataSource::new(path.clone()));
    let projection = Some(ScanProjection::new(
        ["station_name".to_string(), "measurement".to_string()].to_vec(),
    ));

    let scan = Arc::new(Scan::new(data_source, projection));

    let filter = Arc::new(Filter::new(
        scan,
        eq(column("station_name"), lit_string("tokyo")),
    ));

    let project = Arc::new(Projection::new(
        filter,
        vec![
            alias(column("station_name"), "station"),
            alias(
                multiply(column("measurement"), lit_double(1.2)),
                "value",
            ),
        ],
    ));

    let agg = Arc::new(Aggregate::new(
        project,
        vec![column("station")],
        vec![
            alias(max(column("value")), "max_measurement"),
            alias(min(column("value")), "min_measurement"),
            alias(avg(column("value")), "avg_measurement"),
        ],
    ));

    println!("{}", agg.format(0));
}
*/

pub use dataframe::ExecutionContext;
pub use logical_plan::{col, eq, lit};

use crate::logical_plan::{alias, avg, gteq, max, min, multiply};

fn main() {
    let path = "data/weather_stations_small.parquet".to_string();
    let df = ExecutionContext::parquet(path, None)
        // WANT TO WRITE: col("station_name").eq("Tokyo")
        .filter(eq(col("station_name"), lit("Tokyo")))
        .project(vec![
            // WANT TO WRITE: col("station_name").alias("station"),
            alias(col("station_name"), "station"),
            // WANT TO WRITE: (col("measurement") * lit(1.2 as f64)).alias("value")
            alias(multiply(col("measurement"), lit(1.2 as f64)), "value"),
        ])
        .agg(
            vec![col("station")],
            vec![
                alias(max(col("value")), "max_value"),
                alias(min(col("value")), "min_value"),
                alias(avg(col("value")), "avg_value"),
            ],
        )
        .filter(gteq(col("min_value"), lit(2.56 as f64)));

    df.print_logical_plan();
}
