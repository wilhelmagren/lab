pub mod context;
pub mod data_source;
pub mod dataframe;
pub mod logical;
pub mod optimizer;
pub mod physical;
pub mod planner;
pub mod scalar;
pub mod sql;

use crate::context::SessionContext;
use crate::logical::expr::{lit, sum};
use crate::logical::{
    expr::col,
    plan::{JoinKey, JoinType},
};

fn main() {
    // Find passengers on the same ticket where one survived and one did not.
    let ctx = SessionContext::new();

    let passengers = ctx.parquet("data/titanic.parquet");

    /*
    let survivors = passengers.filter(col("Survived").eq(1)).project(vec![
        col("Ticket"),
        col("Name").alias("survivor_name"),
        col("Ticket").alias("survivor_ticket"),
    ]);

    let non_survivors = passengers.filter(col("Survived").eq(0)).project(vec![
        col("Ticket"),
        col("Name").alias("non_survivor_name"),
        col("Ticket").alias("non_survivor_ticket"),
    ]);

    let df = survivors
        .join(
            &non_survivors,
            JoinType::Inner,
            vec![JoinKey::new("Ticket", "Ticket")],
        )
        .limit(23);
    */

    let df = passengers
        .select(vec![col("Name"), col("Ticket"), col("Survived")])
        .filter(col("Survived").eq(lit(1 as i64)))
        .select(vec![col("Name"), col("Ticket")])
        .limit(5);

    for batch in ctx.execute(&df) {
        println!("{:?}", batch);
    }
}
