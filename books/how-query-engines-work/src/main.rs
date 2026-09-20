use arrow::array::RecordBatch;
use arrow::util::pretty::pretty_format_batches;

use tqe::context::SessionContext;
use tqe::logical::expr::asc;
use tqe::logical::expr::avg;
use tqe::logical::expr::col;
use tqe::logical::expr::count;
use tqe::logical::expr::desc;
use tqe::logical::expr::max;
use tqe::logical::expr::min;
use tqe::logical::plan::JoinKey;
use tqe::logical::plan::JoinType;

fn main() {
    let ctx = SessionContext::new();

    let df_users = ctx
        .parquet("./data/users.parquet", "users")
        .filter(col("age").gteq(100i64));

    let df_jobs = ctx.parquet("./data/jobs.parquet", "jobs");

    let df = df_users
        .join(&df_jobs, JoinType::Right, vec![JoinKey::new("id", "id")])
        .agg(
            vec![col("id"), col("name")],
            vec![
                avg(col("salary")).alias("avg_salary"),
                min(col("salary")).alias("min_salary"),
                max(col("salary")).alias("max_salary"),
            ],
        )
        .sort(vec![asc(col("name")), desc(col("avg_salary"))])
        .limit(10);

    // everything is lazy, nothing runs until we do .collect() below...

    println!(
        "{}",
        pretty_format_batches(&ctx.execute(&df).collect::<Vec<RecordBatch>>()).unwrap()
    );

    let df = ctx
        .parquet(
            "./data/weather_stations_small.parquet",
            "weather_stations_small",
        )
        .agg(
            vec![col("station_name")],
            vec![count(col("station_name")).alias("occurrences")],
        )
        .filter(col("occurrences").gt(1 as u64))
        .sort(vec![desc(col("occurrences"))])
        .limit(10);

    println!(
        "{}",
        pretty_format_batches(&ctx.execute(&df).collect::<Vec<RecordBatch>>()).unwrap()
    );

    let df = ctx
        .sql(
            r#"
        SELECT
            station_name,
            min(measurement) AS min_measurement,
            max(measurement) AS max_measurement,
            avg(measurement) AS avg_measurement
        FROM weather_stations_small
        GROUP BY station_name
        ORDER BY station_name
        "#,
        )
        .limit(10);

    println!(
        "{}",
        pretty_format_batches(&ctx.execute(&df).collect::<Vec<RecordBatch>>()).unwrap()
    );
}
