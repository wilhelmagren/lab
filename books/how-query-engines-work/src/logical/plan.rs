use std::collections::HashSet;
use std::sync::Arc;

use arrow::datatypes::{Schema, SchemaRef};

use crate::{data_source::SourceId, logical::expr::LogicalExpr};

#[derive(Clone)]
pub enum LogicalPlanKind {
    Limit(LimitPlan),
    Scan(ScanPlan),
    Filter(FilterPlan),
    Projection(ProjectionPlan),
    Aggregate(AggregatePlan),
    Join(JoinPlan),
}

#[derive(Clone)]
pub struct LogicalPlan(Arc<LogicalPlanKind>);

impl LogicalPlan {
    pub fn new(plan: LogicalPlanKind) -> Self {
        Self(Arc::new(plan))
    }

    pub fn limit(self, limit: usize) -> Self {
        Self::new(LogicalPlanKind::Limit(LimitPlan::new(self, limit)))
    }

    pub fn scan(source_id: SourceId, source_name: impl Into<String>, schema: SchemaRef) -> Self {
        Self::new(LogicalPlanKind::Scan(ScanPlan::new(
            source_id,
            source_name.into(),
            schema,
            None,
        )))
    }

    pub fn filter(self, predicate: LogicalExpr) -> Self {
        Self::new(LogicalPlanKind::Filter(FilterPlan::new(self, predicate)))
    }

    pub fn projection(self, exprs: Vec<LogicalExpr>) -> Self {
        Self::new(LogicalPlanKind::Projection(ProjectionPlan::new(
            self, exprs,
        )))
    }

    pub fn aggregate(self, group_exprs: Vec<LogicalExpr>, agg_exprs: Vec<LogicalExpr>) -> Self {
        Self::new(LogicalPlanKind::Aggregate(AggregatePlan::new(
            self,
            group_exprs,
            agg_exprs,
        )))
    }

    pub fn join(self, right: LogicalPlan, how: JoinType, on: Vec<JoinKey>) -> Self {
        Self::new(LogicalPlanKind::Join(JoinPlan::new(self, right, how, on)))
    }

    pub fn kind(&self) -> &LogicalPlanKind {
        self.0.as_ref()
    }

    pub fn schema(&self) -> &SchemaRef {
        match self.kind() {
            LogicalPlanKind::Limit(lp) => lp.schema(),
            LogicalPlanKind::Scan(sp) => sp.schema(),
            LogicalPlanKind::Filter(fp) => fp.schema(),
            LogicalPlanKind::Projection(pp) => pp.schema(),
            LogicalPlanKind::Aggregate(ap) => ap.schema(),
            LogicalPlanKind::Join(jp) => jp.schema(),
        }
    }

    pub fn inputs(&self) -> Vec<&LogicalPlan> {
        match self.kind() {
            LogicalPlanKind::Limit(lp) => vec![&lp.input],
            LogicalPlanKind::Scan(_) => vec![],
            LogicalPlanKind::Filter(fp) => vec![&fp.input],
            LogicalPlanKind::Projection(pp) => vec![&pp.input],
            LogicalPlanKind::Aggregate(ap) => vec![&ap.input],
            LogicalPlanKind::Join(jp) => vec![&jp.left, &jp.right],
        }
    }

    pub fn format(&self, indent: usize) -> String {
        let mut s = String::new();
        s.push_str(&"  ".repeat(indent));
        s.push_str(self.to_string().as_str());
        s.push_str("\n");
        self.inputs()
            .iter()
            .for_each(|i| s.push_str(i.format(indent + 1).as_str()));
        s
    }
}

impl std::fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            LogicalPlanKind::Limit(plan) => plan.fmt(f),
            LogicalPlanKind::Scan(plan) => plan.fmt(f),
            LogicalPlanKind::Filter(plan) => plan.fmt(f),
            LogicalPlanKind::Projection(plan) => plan.fmt(f),
            LogicalPlanKind::Aggregate(plan) => plan.fmt(f),
            LogicalPlanKind::Join(plan) => plan.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct LimitPlan {
    input: LogicalPlan,
    limit: usize,
    schema: SchemaRef,
}

impl LimitPlan {
    pub fn new(input: LogicalPlan, limit: usize) -> Self {
        let schema = input.schema().clone();
        Self {
            input,
            limit,
            schema,
        }
    }

    pub fn input(&self) -> &LogicalPlan {
        &self.input
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl std::fmt::Display for LimitPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Limit: {}", self.limit)
    }
}

impl From<LimitPlan> for LogicalPlan {
    fn from(value: LimitPlan) -> Self {
        Self(Arc::new(LogicalPlanKind::Limit(value)))
    }
}

#[derive(Clone)]
pub struct ScanPlan {
    source_id: SourceId,
    source_name: String,
    schema: SchemaRef,
    projection: Option<Vec<usize>>,
}

impl ScanPlan {
    pub fn new(
        source_id: SourceId,
        source_name: impl Into<String>,
        schema: SchemaRef,
        projection: Option<Vec<usize>>,
    ) -> Self {
        Self {
            source_id,
            source_name: source_name.into(),
            schema,
            projection,
        }
    }

    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn projection(&self) -> Option<&[usize]> {
        self.projection.as_deref()
    }

    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl std::fmt::Display for ScanPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.projection {
            Some(p) => write!(f, "Scan: {}; projection={:?}", self.source_name, p),
            None => write!(f, "Scan: {}; projection=None", self.source_name),
        }
    }
}

#[derive(Clone)]
pub struct FilterPlan {
    input: LogicalPlan,
    schema: SchemaRef,
    predicate: LogicalExpr,
}

impl FilterPlan {
    pub fn new(input: LogicalPlan, predicate: LogicalExpr) -> Self {
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

    pub fn predicate(&self) -> &LogicalExpr {
        &self.predicate
    }

    pub fn input(&self) -> &LogicalPlan {
        &self.input
    }
}

impl std::fmt::Display for FilterPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter: {}", self.predicate)
    }
}

impl From<FilterPlan> for LogicalPlan {
    fn from(value: FilterPlan) -> Self {
        Self(Arc::new(LogicalPlanKind::Filter(value)))
    }
}

#[derive(Clone)]
pub struct ProjectionPlan {
    input: LogicalPlan,
    exprs: Vec<LogicalExpr>,
    schema: SchemaRef,
}

impl ProjectionPlan {
    pub fn new(input: LogicalPlan, exprs: Vec<LogicalExpr>) -> Self {
        let schema = Arc::new(Schema::new(
            exprs
                .iter()
                .map(|expr| expr.to_field(&input.schema()))
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

    pub fn exprs(&self) -> &[LogicalExpr] {
        &self.exprs
    }

    pub fn input(&self) -> &LogicalPlan {
        &self.input
    }
}

impl std::fmt::Display for ProjectionPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Projection: {}",
            self.exprs
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone)]
pub struct AggregatePlan {
    input: LogicalPlan,
    group_exprs: Vec<LogicalExpr>,
    agg_exprs: Vec<LogicalExpr>,
    schema: SchemaRef,
}

impl AggregatePlan {
    pub fn new(
        input: LogicalPlan,
        group_exprs: Vec<LogicalExpr>,
        agg_exprs: Vec<LogicalExpr>,
    ) -> Self {
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

    pub fn input(&self) -> &LogicalPlan {
        &self.input
    }
}

impl std::fmt::Display for AggregatePlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Aggregation: groupBy={}, aggExpr={}",
            self.group_exprs
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.agg_exprs
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
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
pub struct JoinPlan {
    left: LogicalPlan,
    right: LogicalPlan,
    how: JoinType,
    on: Vec<JoinKey>,
    schema: SchemaRef,
}

impl JoinPlan {
    fn new(left: LogicalPlan, right: LogicalPlan, how: JoinType, on: Vec<JoinKey>) -> Self {
        let duplicate_keys = on
            .iter()
            .filter(|jk| jk.left == jk.right)
            .map(|jk| jk.left.as_str())
            .collect::<HashSet<&str>>();

        let fields = match how {
            JoinType::Inner | JoinType::Left => left
                .schema()
                .fields()
                .iter()
                .chain(
                    right
                        .schema()
                        .fields()
                        .iter()
                        .filter(|f| !duplicate_keys.contains(f.name().as_str())),
                )
                .cloned()
                .collect::<Vec<_>>(),
            JoinType::Right => left
                .schema()
                .fields()
                .iter()
                .filter(|f| !duplicate_keys.contains(f.name().as_str()))
                .chain(right.schema().fields().iter())
                // this is cheap Arc cloning
                .cloned()
                .collect::<Vec<_>>(),
        };

        let schema = Arc::new(Schema::new(fields));

        Self {
            left,
            right,
            how,
            on,
            schema,
        }
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl std::fmt::Display for JoinPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Join: type={}, on=[{}]",
            self.how,
            self.on
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}
