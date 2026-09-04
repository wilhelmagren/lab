use std::sync::Arc;

use arrow::datatypes::{DataType, Field, FieldRef, Schema};

use crate::scalar::ScalarValue;

#[derive(Clone)]
pub enum LogicalExprKind {
    Column(ColumnLogicalExpr),
    Literal(LiteralLogicalExpr),
    Binary(BinaryLogicalExpr),
    Aggregate(AggregateLogicalExpr),
    Alias(AliasLogicalExpr),
}

#[derive(Clone)]
pub struct LogicalExpr(Arc<LogicalExprKind>);

impl LogicalExpr {
    pub fn new(kind: LogicalExprKind) -> Self {
        Self(Arc::new(kind))
    }

    pub fn kind(&self) -> &LogicalExprKind {
        self.0.as_ref()
    }

    pub fn to_field(&self, input: &Schema) -> FieldRef {
        match self.kind() {
            LogicalExprKind::Column(expr) => expr.to_field(input),
            LogicalExprKind::Literal(expr) => expr.to_field(),
            LogicalExprKind::Binary(expr) => expr.to_field(input),
            LogicalExprKind::Aggregate(expr) => expr.to_field(input),
            LogicalExprKind::Alias(expr) => expr.to_field(input),
        }
    }

    pub fn alias(self, name: impl Into<String>) -> Self {
        alias(self, name)
    }

    pub fn eq(self, rhs: impl Into<LogicalExpr>) -> Self {
        eq(self, rhs.into())
    }

    pub fn neq(self, rhs: impl Into<LogicalExpr>) -> Self {
        neq(self, rhs.into())
    }

    pub fn gt(self, rhs: impl Into<LogicalExpr>) -> Self {
        gt(self, rhs.into())
    }

    pub fn gteq(self, rhs: impl Into<LogicalExpr>) -> Self {
        gteq(self, rhs.into())
    }

    pub fn lt(self, rhs: impl Into<LogicalExpr>) -> Self {
        lt(self, rhs.into())
    }

    pub fn lteq(self, rhs: impl Into<LogicalExpr>) -> Self {
        lteq(self, rhs.into())
    }

    pub fn and(self, rhs: impl Into<LogicalExpr>) -> Self {
        and(self, rhs.into())
    }

    pub fn or(self, rhs: impl Into<LogicalExpr>) -> Self {
        or(self, rhs.into())
    }
}

impl std::fmt::Display for LogicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            LogicalExprKind::Column(expr) => expr.fmt(f),
            LogicalExprKind::Literal(expr) => expr.fmt(f),
            LogicalExprKind::Binary(expr) => expr.fmt(f),
            LogicalExprKind::Aggregate(expr) => expr.fmt(f),
            LogicalExprKind::Alias(expr) => expr.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct ColumnLogicalExpr {
    name: String,
}

impl ColumnLogicalExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        input
            .fields()
            .find(&self.name)
            .unwrap_or_else(|| panic!("column '{}' does not exist", self.name))
            .1
            .clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for ColumnLogicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.name)
    }
}

/// Create a column expression.
pub fn col(name: impl Into<String>) -> LogicalExpr {
    LogicalExpr::new(LogicalExprKind::Column(ColumnLogicalExpr {
        name: name.into(),
    }))
}

#[derive(Clone)]
pub struct LiteralLogicalExpr {
    value: ScalarValue,
}

impl LiteralLogicalExpr {
    pub fn value(&self) -> &ScalarValue {
        &self.value
    }

    fn to_field(&self) -> FieldRef {
        Arc::new(Field::new(
            self.value.to_string(),
            self.value.data_type(),
            false,
        ))
    }
}

impl std::fmt::Display for LiteralLogicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn lit(value: impl Into<ScalarValue>) -> LogicalExpr {
    LogicalExpr::new(LogicalExprKind::Literal(LiteralLogicalExpr {
        value: value.into(),
    }))
}

impl From<ScalarValue> for LogicalExpr {
    fn from(value: ScalarValue) -> Self {
        LogicalExpr::new(LogicalExprKind::Literal(LiteralLogicalExpr { value }))
    }
}

macro_rules! impl_expr_from_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for LogicalExpr {
                fn from(value: $ty) -> Self {
                    lit(value)
                }
            }
        )*
    };
}

impl_expr_from_scalar!(bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String);

impl From<&str> for LogicalExpr {
    fn from(value: &str) -> Self {
        lit(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // math
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,

    // comp
    Eq,
    NotEq,
    Gt,
    GtEq,
    Lt,
    LtEq,

    // boolean
    And,
    Or,
}

impl BinaryOp {
    pub fn result_data_type(&self, left: &DataType, _right: &DataType) -> DataType {
        match self {
            Self::Eq
            | Self::NotEq
            | Self::Gt
            | Self::GtEq
            | Self::Lt
            | Self::LtEq
            | Self::And
            | Self::Or => DataType::Boolean,
            Self::Add | Self::Subtract | Self::Multiply | Self::Divide | Self::Modulus => {
                left.clone()
            }
        }
    }
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Modulus => "%",

            Self::Eq => "=",
            Self::NotEq => "!=",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::Lt => "<",
            Self::LtEq => "<=",

            Self::And => "AND",
            Self::Or => "OR",
        };

        f.write_str(symbol)
    }
}

#[derive(Clone)]
pub struct BinaryLogicalExpr {
    name: String,
    left: LogicalExpr,
    op: BinaryOp,
    right: LogicalExpr,
}

impl BinaryLogicalExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        let lf = self.left.to_field(input);
        let rf = self.right.to_field(input);

        Arc::new(Field::new(
            &self.name,
            self.op.result_data_type(lf.data_type(), rf.data_type()),
            lf.is_nullable() || rf.is_nullable(),
        ))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn left(&self) -> &LogicalExpr {
        &self.left
    }

    pub fn op(&self) -> &BinaryOp {
        &self.op
    }

    pub fn right(&self) -> &LogicalExpr {
        &self.right
    }
}

impl std::fmt::Display for BinaryLogicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

fn binary(
    name: impl Into<String>,
    left: LogicalExpr,
    op: BinaryOp,
    right: LogicalExpr,
) -> LogicalExpr {
    LogicalExpr::new(LogicalExprKind::Binary(BinaryLogicalExpr {
        name: name.into(),
        left,
        op,
        right,
    }))
}

pub fn eq(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("eq", left, BinaryOp::Eq, right)
}

pub fn neq(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("neq", left, BinaryOp::NotEq, right)
}

pub fn gt(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("gt", left, BinaryOp::Gt, right)
}

pub fn gteq(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("gteq", left, BinaryOp::GtEq, right)
}

pub fn lt(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("lt", left, BinaryOp::Lt, right)
}

pub fn lteq(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("lteq", left, BinaryOp::LtEq, right)
}

pub fn and(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("and", left, BinaryOp::And, right)
}

pub fn or(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("or", left, BinaryOp::Or, right)
}

pub fn add(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("add", left, BinaryOp::Add, right)
}

pub fn subtract(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("subtract", left, BinaryOp::Subtract, right)
}

pub fn multiply(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("multiply", left, BinaryOp::Multiply, right)
}

pub fn divide(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("divide", left, BinaryOp::Divide, right)
}

pub fn modulus(left: LogicalExpr, right: LogicalExpr) -> LogicalExpr {
    binary("modulus", left, BinaryOp::Modulus, right)
}

impl<Rhs> std::ops::Add<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn add(self, rhs: Rhs) -> Self::Output {
        add(self, rhs.into())
    }
}

impl<Rhs> std::ops::Sub<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn sub(self, rhs: Rhs) -> Self::Output {
        subtract(self, rhs.into())
    }
}

impl<Rhs> std::ops::Mul<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn mul(self, rhs: Rhs) -> Self::Output {
        multiply(self, rhs.into())
    }
}

impl<Rhs> std::ops::Div<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn div(self, rhs: Rhs) -> Self::Output {
        divide(self, rhs.into())
    }
}

impl<Rhs> std::ops::Rem<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn rem(self, rhs: Rhs) -> Self::Output {
        modulus(self, rhs.into())
    }
}

// we can not overload '&&' or '||' in Rust so we use these instead
impl<Rhs> std::ops::BitAnd<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn bitand(self, rhs: Rhs) -> Self::Output {
        and(self, rhs.into())
    }
}

impl<Rhs> std::ops::BitOr<Rhs> for LogicalExpr
where
    Rhs: Into<LogicalExpr>,
{
    type Output = LogicalExpr;
    fn bitor(self, rhs: Rhs) -> Self::Output {
        or(self, rhs.into())
    }
}

#[derive(Clone)]
pub enum AggregateOp {
    Min,
    Max,
    Avg,
    Sum,
    Count,
}

impl std::fmt::Display for AggregateOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::Avg => "AVG",
            Self::Sum => "SUM",
            Self::Count => "COUNT",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct AggregateLogicalExpr {
    name: String,
    op: AggregateOp,
    pub expr: LogicalExpr,
}

impl AggregateLogicalExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        match self.op {
            AggregateOp::Count => Arc::new(Field::new(self.op.to_string(), DataType::Int32, false)),
            _ => {
                let f = self.expr.to_field(input);
                Arc::new(Field::new(
                    &self.name,
                    f.data_type().clone(),
                    f.is_nullable(),
                ))
            }
        }
    }
}

impl std::fmt::Display for AggregateLogicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.op, self.expr)
    }
}

fn aggregate(name: impl Into<String>, op: AggregateOp, expr: LogicalExpr) -> LogicalExpr {
    LogicalExpr::new(LogicalExprKind::Aggregate(AggregateLogicalExpr {
        name: name.into(),
        op,
        expr,
    }))
}

pub fn min(expr: LogicalExpr) -> LogicalExpr {
    aggregate("min", AggregateOp::Min, expr)
}

pub fn max(expr: LogicalExpr) -> LogicalExpr {
    aggregate("max", AggregateOp::Max, expr)
}

pub fn avg(expr: LogicalExpr) -> LogicalExpr {
    aggregate("avg", AggregateOp::Avg, expr)
}

pub fn sum(expr: LogicalExpr) -> LogicalExpr {
    aggregate("sum", AggregateOp::Sum, expr)
}

pub fn count(expr: LogicalExpr) -> LogicalExpr {
    aggregate("count", AggregateOp::Count, expr)
}

#[derive(Clone)]
pub struct AliasLogicalExpr {
    expr: LogicalExpr,
    name: String,
}

impl AliasLogicalExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        let f = self.expr.to_field(input);
        Arc::new(Field::new(
            &self.name,
            f.data_type().clone(),
            f.is_nullable(),
        ))
    }

    pub fn expr(&self) -> &LogicalExpr {
        &self.expr
    }
}

impl std::fmt::Display for AliasLogicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} AS {}", self.expr, self.name)
    }
}

pub fn alias(expr: LogicalExpr, name: impl Into<String>) -> LogicalExpr {
    LogicalExpr::new(LogicalExprKind::Alias(AliasLogicalExpr {
        expr,
        name: name.into(),
    }))
}
