use std::sync::Arc;

use arrow::datatypes::{Schema, SchemaRef};

use crate::logical::expr::Expr;

#[derive(Clone)]
pub enum LogicalPlanKind {
    Scan(Scan),
    Filter(Filter),
    Project(Project),
    Aggregate(Aggregate),
    Join(Join),
}

#[derive(Clone)]
pub struct LogicalPlan(Arc<LogicalPlanKind>);

impl LogicalPlan {
    pub fn kind(&self) -> &LogicalPlanKind {
        self.0.as_ref()
    }

    pub fn schema(&self) -> &Schema {
        match self.kind() {
            LogicalPlanKind::Scan(s) => todo!(),
            _ => todo!(),
        }
    }
}

#[derive(Clone)]
struct Scan {
    // scan has no input because this is always a leaf node in the AST
    path: String,
    schema: SchemaRef,
    projection: Option<Vec<String>>,
}

#[derive(Clone)]
struct Filter {
    input: LogicalPlan,
    predicates: Expr,
}

#[derive(Clone)]
struct Project {
    input: LogicalPlan,
    exprs: Vec<Expr>,
    schema: SchemaRef,
}

impl Project {
    pub fn new(input: LogicalPlan, exprs: Vec<Expr>) -> Self {
        let input_schema = input.schema();
        let schema = Arc::new(Schema::new(
            exprs
                .iter()
                .map(|expr| expr.to_field(input_schema.as_ref()))
                .collect::<Vec<_>>(),
        ));

        Self {
            input,
            exprs,
            schema,
        }
    }
}

#[derive(Clone)]
struct Aggregate {
    input: LogicalPlan,
    group_exprs: Vec<Expr>,
    agg_exprs: Vec<Expr>,
}

#[derive(Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Inner => "inner",
            Self::Left => "left",
            Self::Right => "right",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct JoinKey {
    pub left: String,
    pub right: String,
}

impl JoinKey {
    pub fn new(left: impl Into<String>, right: impl Into<String>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl std::fmt::Display for JoinKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.left, self.right)
    }
}

#[derive(Clone)]
struct Join {
    left: LogicalPlan,
    right: LogicalPlan,
    how: JoinType,
    on: Vec<JoinKey>,
}
