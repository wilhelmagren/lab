use std::sync::Arc;

use arrow::datatypes::{Schema, SchemaRef};

use crate::logical::expr::Expr;

#[derive(Clone)]
pub enum LogicalPlanKind {
    Scan(ScanPlan),
    Filter(FilterPlan),
    Project(ProjectPlan),
    Aggregate(AggregatePlan),
    Join(JoinPlan),
}

#[derive(Clone)]
pub struct LogicalPlan(Arc<LogicalPlanKind>);

impl LogicalPlan {
    pub fn kind(&self) -> &LogicalPlanKind {
        self.0.as_ref()
    }

    pub fn schema(&self) -> &SchemaRef {
        match self.kind() {
            LogicalPlanKind::Scan(sp) => todo!(),
            LogicalPlanKind::Filter(fp) => fp.schema(),
            LogicalPlanKind::Project(pp) => pp.schema(),
            LogicalPlanKind::Aggregate(ap) => ap.schema(),
            LogicalPlanKind::Join(jp) => jp.schema(),
        }
    }
}

#[derive(Clone)]
struct ScanPlan {
    // scan has no input because this is always a leaf node in the AST
    path: String,
    projection: Option<Vec<String>>,
    schema: SchemaRef,
}

#[derive(Clone)]
struct FilterPlan {
    input: LogicalPlan,
    predicate: Expr,
    schema: SchemaRef,
}

impl FilterPlan {
    pub fn new(input: LogicalPlan, predicate: Expr) -> Self {
        let schema = input.schema().clone();
        Self {
            input,
            predicate,
            schema,
        }
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl std::fmt::Display for FilterPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter: {}", self.predicate)
    }
}

#[derive(Clone)]
struct ProjectPlan {
    input: LogicalPlan,
    exprs: Vec<Expr>,
    schema: SchemaRef,
}

impl ProjectPlan {
    pub fn new(input: LogicalPlan, exprs: Vec<Expr>) -> Self {
        let input_schema = input.schema();
        let schema = Arc::new(Schema::new(
            exprs
                .iter()
                .map(|expr| expr.to_field(input_schema))
                .collect::<Vec<_>>(),
        ));

        Self {
            input,
            exprs,
            schema,
        }
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

#[derive(Clone)]
struct AggregatePlan {
    input: LogicalPlan,
    group_exprs: Vec<Expr>,
    agg_exprs: Vec<Expr>,
    schema: SchemaRef,
}

impl AggregatePlan {
    pub fn new(input: LogicalPlan, group_exprs: Vec<Expr>, agg_exprs: Vec<Expr>) -> Self {
        let input_schema = input.schema();
        let schema = Arc::new(Schema::new(
            group_exprs
                .iter()
                .map(|expr| expr.to_field(input_schema))
                .chain(agg_exprs.iter().map(|expr| expr.to_field(input_schema)))
                .collect::<Vec<_>>(),
        ));

        Self {
            input,
            group_exprs,
            agg_exprs,
            schema,
        }
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
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
struct JoinPlan {
    left: LogicalPlan,
    right: LogicalPlan,
    how: JoinType,
    on: Vec<JoinKey>,
    schema: SchemaRef,
}

impl JoinPlan {
    fn new(left: LogicalPlan, right: LogicalPlan, how: JoinType, on: JoinKey) -> Self {
        todo!()
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}
