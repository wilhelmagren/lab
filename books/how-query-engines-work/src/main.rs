use arrow::array::RecordBatch;
use arrow::util::pretty::pretty_format_batches;

use tqe::context::SessionContext;
use tqe::logical::expr::avg;
use tqe::logical::expr::col;
use tqe::logical::expr::max;
use tqe::logical::expr::min;
use tqe::logical::plan::JoinKey;
use tqe::logical::plan::JoinType;

fn main() {
    let ctx = SessionContext::new();

    let df_users = ctx
        .parquet("./data/users.parquet")
        .filter(col("age").gteq(100i64));

    let df_jobs = ctx.parquet("./data/jobs.parquet");

    let df = df_users
        .join(&df_jobs, JoinType::Inner, vec![JoinKey::new("id", "id")])
        .agg(
            vec![col("id"), col("name")],
            vec![
                avg(col("salary")).alias("avg_salary"),
                min(col("salary")).alias("min_salary"),
                max(col("salary")).alias("max_salary"),
            ],
        )
        .limit(10);

    // everything is lazy, nothing runs until we do .collect() below...

    println!(
        "{}",
        pretty_format_batches(&ctx.execute(&df).collect::<Vec<RecordBatch>>()).unwrap()
    );
}
