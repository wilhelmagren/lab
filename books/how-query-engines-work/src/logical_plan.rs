use std::sync::Arc;

use arrow::datatypes::{DataType, Field, FieldRef};

use crate::{
    array::ScalarValue, data_source::DataSourceRef, data_source::ScanProjection, schema::Schema,
    schema::SchemaRef,
};

pub type LogicalPlanRef = Arc<dyn LogicalPlan>;

/// A logical plan represents a data transformation or action that returns a relation.
pub trait LogicalPlan: std::fmt::Display {
    /// Returns the schema of the data that will be produced by this logical plan.
    fn schema(&self) -> SchemaRef;
    /// Returns the children (inputs) of this logical plan. This method enables use of
    /// the visitor pattern to walk a query tree easily.
    fn children(&self) -> Option<Vec<LogicalPlanRef>>;
    /// Format the logical plan in human readable form.
    fn format(&self, indent: usize) -> String {
        let mut s = String::new();
        (0..indent).for_each(|_| s.push_str("  "));
        s.push_str(&self.to_string());
        s.push_str("\n");
        if let Some(children) = self.children() {
            children
                .into_iter()
                .for_each(|c| s.push_str(&c.format(indent + 1)));
        }
        s
    }
}

/*
          Expression type                 Examples
        ----------------------       -------------------------
         Literal Value	            "hello", 12.34, true
         Column Reference       	user_id, first_name, salary
         Math Expression	        salary * 0.1, price + tax
         Comparison Expression	    age >= 21, status != 'inactive'
         Boolean Expression	        age >= 21 AND country = 'US'
         Aggregate Expression	    MIN(salary), MAX(salary), SUM(amount), COUNT(*)
         Scalar Function	        UPPER(name), CONCAT(first_name, ' ', last_name)
         Aliased Expression	        salary * 1.1 AS new_salary
*/

pub type LogicalExprRef = Arc<dyn LogicalExpr>;

/// Loigcal Expression for use in logical query plans. The logical expression provides
/// information needed during the planning phase such as the name and dtype of the exprs.
pub trait LogicalExpr: std::fmt::Display {
    /// Return metadata about the value that will be produced by the this expression when evaluated against a particular input.
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef;
}

pub struct ColumnExpr {
    name: String,
}

impl std::fmt::Display for ColumnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", &self.name)
    }
}

impl LogicalExpr for ColumnExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        // clone is cheap on arc :)))
        input.schema().fields().find(&self.name).unwrap().1.clone()
    }
}

pub fn column(name: impl Into<String>) -> LogicalExprRef {
    Arc::new(ColumnExpr { name: name.into() })
}

pub fn col(name: impl Into<String>) -> LogicalExprRef {
    Arc::new(ColumnExpr { name: name.into() })
}

pub struct LiteralExpr {
    value: ScalarValue,
}

impl std::fmt::Display for LiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.value)
    }
}

impl LogicalExpr for LiteralExpr {
    fn to_field(&self, _input: LogicalPlanRef) -> FieldRef {
        // a literal value that is not a NULL literal can never be NULL :PP
        Arc::new(Field::new(
            &self.value.to_string(),
            self.value.dtype(),
            false,
        ))
    }
}

pub fn lit<T>(t: T) -> LogicalExprRef
where
    T: Into<ScalarValue>,
{
    Arc::new(LiteralExpr { value: t.into() })
}

pub fn lit_string(s: impl Into<String>) -> LogicalExprRef {
    Arc::new(LiteralExpr {
        value: ScalarValue::Utf8(s.into()),
    })
}

pub fn lit_long(n: i64) -> LogicalExprRef {
    Arc::new(LiteralExpr {
        value: ScalarValue::Int64(n),
    })
}

pub fn lit_double(n: f64) -> LogicalExprRef {
    Arc::new(LiteralExpr {
        value: ScalarValue::Float64(n),
    })
}

pub enum BooleanOp {
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,
    And,
    Or,
}

impl std::fmt::Display for BooleanOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BooleanOp::Eq => write!(f, "="),
            BooleanOp::Neq => write!(f, "!="),
            BooleanOp::Gt => write!(f, ">"),
            BooleanOp::GtEq => write!(f, ">="),
            BooleanOp::Lt => write!(f, "<"),
            BooleanOp::LtEq => write!(f, "<="),
            BooleanOp::And => write!(f, "AND"),
            BooleanOp::Or => write!(f, "OR"),
        }
    }
}

pub struct BooleanBinaryExpr {
    name: String,
    op: BooleanOp,
    l: LogicalExprRef,
    r: LogicalExprRef,
}

impl LogicalExpr for BooleanBinaryExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        let lf = self.l.to_field(input.clone());
        let rf = self.r.to_field(input);
        Arc::new(Field::new(
            &self.name,
            DataType::Boolean,
            lf.is_nullable() || rf.is_nullable(),
        ))
    }
}

impl std::fmt::Display for BooleanBinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.l, self.op, self.r)
    }
}

pub fn eq(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "eq".into(),
        op: BooleanOp::Eq,
        l,
        r,
    })
}

pub fn neq(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "neq".into(),
        op: BooleanOp::Neq,
        l,
        r,
    })
}

pub fn gt(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "gt".into(),
        op: BooleanOp::Gt,
        l,
        r,
    })
}

pub fn gteq(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "gteq".into(),
        op: BooleanOp::GtEq,
        l,
        r,
    })
}

pub fn lt(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "lt".into(),
        op: BooleanOp::Lt,
        l,
        r,
    })
}

pub fn lteq(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "lteq".into(),
        op: BooleanOp::LtEq,
        l,
        r,
    })
}

pub fn and(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "and".into(),
        op: BooleanOp::And,
        l,
        r,
    })
}

pub fn or(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(BooleanBinaryExpr {
        name: "or".into(),
        op: BooleanOp::Or,
        l,
        r,
    })
}

pub enum MathOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

impl std::fmt::Display for MathOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            MathOp::Add => "+",
            MathOp::Subtract => "-",
            MathOp::Multiply => "*",
            MathOp::Divide => "/",
            MathOp::Modulus => "%",
        };
        write!(f, "{}", symbol)
    }
}

pub struct MathExpr {
    name: String,
    op: MathOp,
    l: LogicalExprRef,
    r: LogicalExprRef,
}

impl LogicalExpr for MathExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        let lf = self.l.to_field(input);
        Arc::new(Field::new(
            &self.name,
            lf.data_type().clone(),
            lf.is_nullable(),
        ))
    }
}

impl std::fmt::Display for MathExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.l, self.op, self.r)
    }
}

pub fn add(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(MathExpr {
        name: "add".into(),
        op: MathOp::Add,
        l,
        r,
    })
}

pub fn subtract(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(MathExpr {
        name: "subtract".into(),
        op: MathOp::Subtract,
        l,
        r,
    })
}

pub fn multiply(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(MathExpr {
        name: "multiplty".into(),
        op: MathOp::Multiply,
        l,
        r,
    })
}

pub fn divide(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(MathExpr {
        name: "divide".into(),
        op: MathOp::Divide,
        l,
        r,
    })
}

pub fn modulus(l: LogicalExprRef, r: LogicalExprRef) -> LogicalExprRef {
    Arc::new(MathExpr {
        name: "modulus".into(),
        op: MathOp::Modulus,
        l,
        r,
    })
}

pub enum AggregateOp {
    Sum,
    Min,
    Max,
    Avg,
    Count,
}

impl std::fmt::Display for AggregateOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            AggregateOp::Sum => "SUM",
            AggregateOp::Min => "MIN",
            AggregateOp::Max => "MAX",
            AggregateOp::Avg => "AVG",
            AggregateOp::Count => "COUNT",
        };
        write!(f, "{}", symbol)
    }
}

pub struct AggregateExpr {
    op: AggregateOp,
    expr: LogicalExprRef,
}

impl LogicalExpr for AggregateExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        match self.op {
            AggregateOp::Count => Arc::new(Field::new(self.op.to_string(), DataType::Int32, false)),
            _ => {
                let f = self.expr.to_field(input);
                Arc::new(Field::new(
                    self.op.to_string(),
                    f.data_type().clone(),
                    f.is_nullable(),
                ))
            }
        }
    }
}

impl std::fmt::Display for AggregateExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", &self.op, self.expr)
    }
}

pub fn sum(input: LogicalExprRef) -> LogicalExprRef {
    Arc::new(AggregateExpr {
        op: AggregateOp::Sum,
        expr: input,
    })
}

pub fn min(input: LogicalExprRef) -> LogicalExprRef {
    Arc::new(AggregateExpr {
        op: AggregateOp::Min,
        expr: input,
    })
}

pub fn max(input: LogicalExprRef) -> LogicalExprRef {
    Arc::new(AggregateExpr {
        op: AggregateOp::Max,
        expr: input,
    })
}

pub fn avg(input: LogicalExprRef) -> LogicalExprRef {
    Arc::new(AggregateExpr {
        op: AggregateOp::Avg,
        expr: input,
    })
}

pub fn count(input: LogicalExprRef) -> LogicalExprRef {
    Arc::new(AggregateExpr {
        op: AggregateOp::Count,
        expr: input,
    })
}

pub struct AliasExpr {
    alias: String,
    expr: LogicalExprRef,
}

impl std::fmt::Display for AliasExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} AS {}", self.expr, &self.alias)
    }
}

impl LogicalExpr for AliasExpr {
    fn to_field(&self, input: LogicalPlanRef) -> FieldRef {
        let f = self.expr.to_field(input);
        Arc::new(Field::new(
            &self.alias,
            f.data_type().clone(),
            f.is_nullable(),
        ))
    }
}

pub fn alias(expr: LogicalExprRef, alias: impl Into<String>) -> LogicalExprRef {
    Arc::new(AliasExpr {
        alias: alias.into(),
        expr,
    })
}

////////// logical plans

pub struct Scan {
    path: String,
    schema: SchemaRef,
    projection: Option<ScanProjection>,
}

impl Scan {
    pub fn new(data_source: DataSourceRef, projection: Option<ScanProjection>) -> Self {
        let mut schema = data_source.schema();

        if let Some(proj) = &projection {
            schema = Arc::new(schema.project(proj.indices(schema.clone())));
        };

        Self {
            path: data_source.name().to_string(),
            schema,
            projection,
        }
    }
}

impl LogicalPlan for Scan {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn children(&self) -> Option<Vec<LogicalPlanRef>> {
        None
    }
}

impl std::fmt::Display for Scan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.projection {
            Some(p) => write!(f, "Scan: {}; projection={}", self.path, p),
            None => write!(f, "Scan: {}; projection=None", self.path),
        }
    }
}

pub struct Filter {
    input: LogicalPlanRef,
    expr: LogicalExprRef,
}

impl Filter {
    pub fn new(input: LogicalPlanRef, expr: LogicalExprRef) -> Self {
        Self { input, expr }
    }
}

impl LogicalPlan for Filter {
    fn schema(&self) -> SchemaRef {
        self.input.schema().clone()
    }

    fn children(&self) -> Option<Vec<LogicalPlanRef>> {
        Some(vec![self.input.clone()])
    }
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Filter: {}", self.expr)
    }
}

pub struct Projection {
    input: LogicalPlanRef,
    exprs: Vec<LogicalExprRef>,
}

impl Projection {
    pub fn new(input: LogicalPlanRef, exprs: Vec<LogicalExprRef>) -> Self {
        Self { input, exprs }
    }
}

impl LogicalPlan for Projection {
    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::from_fields(
            self.exprs
                .iter()
                .map(|expr| expr.to_field(self.input.clone())),
        ))
    }

    fn children(&self) -> Option<Vec<LogicalPlanRef>> {
        Some(vec![self.input.clone()])
    }
}

impl std::fmt::Display for Projection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Projection: {}",
            self.exprs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub struct Aggregate {
    input: LogicalPlanRef,
    group_exprs: Vec<LogicalExprRef>,
    agg_exprs: Vec<LogicalExprRef>,
}

impl Aggregate {
    pub fn new(
        input: LogicalPlanRef,
        group_exprs: Vec<LogicalExprRef>,
        agg_exprs: Vec<LogicalExprRef>,
    ) -> Self {
        Self {
            input,
            group_exprs,
            agg_exprs,
        }
    }
}

impl LogicalPlan for Aggregate {
    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::from_fields(
            self.group_exprs
                .iter()
                .map(|expr| expr.to_field(self.input.clone()))
                .chain(
                    self.agg_exprs
                        .iter()
                        .map(|expr| expr.to_field(self.input.clone())),
                ),
        ))
    }

    fn children(&self) -> Option<Vec<LogicalPlanRef>> {
        Some(vec![self.input.clone()])
    }
}

impl std::fmt::Display for Aggregate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Aggregate: groupExpr={}, aggExpr={}",
            self.group_exprs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(","),
            self.agg_exprs
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(","),
        )
    }
}

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

pub struct Join {
    l: LogicalPlanRef,
    r: LogicalPlanRef,
    how: JoinType,
    on: Vec<(String, String)>,
}

impl Join {
    pub fn new(
        l: LogicalPlanRef,
        r: LogicalPlanRef,
        how: JoinType,
        on: Vec<(String, String)>,
    ) -> Self {
        Self { l, r, how, on }
    }
}

impl LogicalPlan for Join {
    fn schema(&self) -> SchemaRef {
        // this depends on the join type, im too lazy to do now
        // TODO: do this xd
        todo!()
    }

    fn children(&self) -> Option<Vec<LogicalPlanRef>> {
        Some(vec![self.l.clone(), self.r.clone()])
    }
}

impl std::fmt::Display for Join {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Join: how={}, on={}",
            self.how,
            self.on
                .iter()
                .map(|(l, r)| format!("{}={}", l, r))
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}
