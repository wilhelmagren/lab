pub mod array;
pub mod data_source;
pub mod logical_plan;
pub mod record_batch;
pub mod schema;

use std::sync::Arc;

use crate::{
    data_source::{ParquetDataSource, ScanProjection},
    logical_plan::{
        Aggregate, Filter, LogicalPlan, Projection, Scan, alias, avg, column, eq, lit_double,
        lit_string, multiply,
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
                "new_measurement",
            ),
        ],
    ));

    let agg = Arc::new(Aggregate::new(
        project,
        vec![column("station")],
        vec![alias(avg(column("new_measurement")), "avg_measurement")],
    ));

    println!("{}", agg.format(0));
}
