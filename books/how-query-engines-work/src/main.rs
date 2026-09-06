use arrow::array::RecordBatch;
use arrow::util::pretty::pretty_format_batches;
use bytes::Bytes;

use tqe::context::SessionContext;
use tqe::logical::expr::avg;
use tqe::logical::expr::col;
use tqe::logical::expr::max;
use tqe::logical::expr::min;
use tqe::logical::plan::JoinKey;
use tqe::logical::plan::JoinType;

fn main() {
    let ctx = SessionContext::new();

    let df_a = ctx.parquet("./data/a.parquet");
    let results: Vec<RecordBatch> = ctx.execute(&df_a).collect();
    println!("{}", pretty_format_batches(&results).unwrap());

    let df_b = ctx.parquet("./data/b.parquet");
    let results: Vec<RecordBatch> = ctx.execute(&df_b).collect();
    println!("{}", pretty_format_batches(&results).unwrap());

    let df = df_a.join(&df_b, JoinType::Inner, vec![JoinKey::new("id", "id")]);
    let results: Vec<RecordBatch> = ctx.execute(&df).collect();
    println!("{}", pretty_format_batches(&results).unwrap());
}
