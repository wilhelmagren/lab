# TQE - Tiny Query Engine

This is what I build from reading the "How Query Engines Work" book by Andy Grove.

- https://howqueryengineswork.com/00-introduction.html
- https://github.com/andygrove/how-query-engines-work

**NO AI ALLOWED HERE, GO AWAY MR CLADUE**


```
                    User API
                       │
                       ▼
                   DataFrame
                       │
                       ▼
              ┌──────────────────┐
              │     logical      │
              │                  │
              │ Expr             │
              │ LogicalPlan      │
              └────────┬─────────┘
                       │
                       ▼
                 optimizer
                       │
                       ▼
              ┌──────────────────┐
              │     physical     │
              │                  │
              │ PhysicalExpr     │
              │ ExecutionPlan    │
              └────────┬─────────┘
                       │
                       ▼
                Arrow kernels
                       │
                       ▼
              Arrow RecordBatch
```

```rust
use crate::context::SessionContext;
use crate::logical::expr::{col, lit};

fn main() {
    let df= SessionContext::new()
        .parquet("data/titanic.parquet")
        .select(vec![col("Name"), col("Ticket"), col("Survived")])
        .filter(col("Survived").eq(lit(1 as i64)))
        .select(vec![col("Name"), col("Ticket")])
        .limit(5);

    for batch in ctx.execute(&df) {
        println!("{:?}", batch);
    }
}
```
