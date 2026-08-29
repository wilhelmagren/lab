//! Logical expression AST.
//!
//! ```text
//! Expr
//! └── Arc<ExprKind>
//!     ├── Column
//!     │   └── name: String
//!     │
//!     ├── Literal
//!     │   └── value: ScalarValue
//!     │
//!     ├── Binary
//!     │   ├── left: Expr
//!     │   ├── op: BinaryOp
//!     │   │   ├── Arithmetic
//!     │   │   │   ├── Add
//!     │   │   │   ├── Subtract
//!     │   │   │   ├── Multiply
//!     │   │   │   ├── Divide
//!     │   │   │   └── Modulus
//!     │   │   │
//!     │   │   ├── Comparison
//!     │   │   │   ├── Eq
//!     │   │   │   ├── NotEq
//!     │   │   │   ├── Gt
//!     │   │   │   ├── GtEq
//!     │   │   │   ├── Lt
//!     │   │   │   └── LtEq
//!     │   │   │
//!     │   │   └── Boolean
//!     │   │       ├── And
//!     │   │       └── Or
//!     │   └── right: Expr
//!     │
//!     ├── Aggregate
//!     │   ├── op: AggregateOp
//!     │   │   ├── Sum
//!     │   │   ├── Min
//!     │   │   ├── Max
//!     │   │   ├── Avg
//!     │   │   └── Count
//!     │   └── expr: Expr
//!     │
//!     └── Alias
//!         ├── expr: Expr
//!         └── name: String
//! ```
//!
//! Expressions form a recursive tree. For example:
//!
//! ```text
//! (col("salary") * 0.1).alias("bonus")
//!
//! Alias
//! ├── expr: Binary
//! │   ├── left: Column("salary")
//! │   ├── op: Multiply
//! │   └── right: Literal(Float64(0.1))
//! └── name: "bonus"
//! ```
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, FieldRef, Schema};

use crate::scalar::ScalarValue;

#[derive(Clone)]
pub enum ExprKind {
    Column(ColumnExpr),
    Literal(LiteralExpr),
    Binary(BinaryExpr),
    Aggregate(AggregateExpr),
    Alias(AliasExpr),
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Self(Arc::new(kind))
    }

    pub fn kind(&self) -> &ExprKind {
        self.0.as_ref()
    }

    pub fn to_field(&self, input: &Schema) -> FieldRef {
        match self.kind() {
            ExprKind::Column(expr) => expr.to_field(input),
            ExprKind::Literal(expr) => expr.to_field(),
            ExprKind::Binary(expr) => expr.to_field(input),
            ExprKind::Aggregate(expr) => expr.to_field(input),
            ExprKind::Alias(expr) => expr.to_field(input),
        }
    }
}

#[derive(Clone)]
pub struct Expr(Arc<ExprKind>);

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            ExprKind::Column(expr) => expr.fmt(f),
            ExprKind::Literal(expr) => expr.fmt(f),
            ExprKind::Binary(expr) => expr.fmt(f),
            ExprKind::Aggregate(expr) => expr.fmt(f),
            ExprKind::Alias(expr) => expr.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct ColumnExpr {
    name: String,
}

impl ColumnExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        input
            .fields()
            .find(&self.name)
            .unwrap_or_else(|| panic!("column '{}' does not exist", self.name))
            .1
            .clone()
    }
}

impl std::fmt::Display for ColumnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Create a column expression.
pub fn col(name: impl Into<String>) -> Expr {
    Expr::new(ExprKind::Column(ColumnExpr { name: name.into() }))
}

#[derive(Clone)]
pub struct LiteralExpr {
    value: ScalarValue,
}

impl LiteralExpr {
    fn to_field(&self) -> FieldRef {
        Arc::new(Field::new(
            self.value.to_string(),
            self.value.data_type(),
            false,
        ))
    }
}

impl std::fmt::Display for LiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub fn lit(value: impl Into<ScalarValue>) -> Expr {
    Expr::new(ExprKind::Literal(LiteralExpr {
        value: value.into(),
    }))
}

impl From<ScalarValue> for Expr {
    fn from(value: ScalarValue) -> Self {
        Expr::new(ExprKind::Literal(LiteralExpr { value }))
    }
}

macro_rules! impl_expr_from_scalar {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Expr {
                fn from(value: $ty) -> Self {
                    lit(value)
                }
            }
        )*
    };
}

impl_expr_from_scalar!(bool, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, String);

impl From<&str> for Expr {
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
    fn result_data_type(&self, left: &DataType, _right: &DataType) -> DataType {
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
pub struct BinaryExpr {
    name: String,
    left: Expr,
    op: BinaryOp,
    right: Expr,
}

impl BinaryExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        let lf = self.left.to_field(input);
        let rf = self.right.to_field(input);

        Arc::new(Field::new(
            &self.name,
            self.op.result_data_type(lf.data_type(), rf.data_type()),
            lf.is_nullable() || rf.is_nullable(),
        ))
    }
}

impl std::fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

fn binary(name: impl Into<String>, left: Expr, op: BinaryOp, right: Expr) -> Expr {
    Expr::new(ExprKind::Binary(BinaryExpr {
        name: name.into(),
        left,
        op,
        right,
    }))
}

pub fn eq(left: Expr, right: Expr) -> Expr {
    binary("eq", left, BinaryOp::Eq, right)
}

pub fn neq(left: Expr, right: Expr) -> Expr {
    binary("neq", left, BinaryOp::NotEq, right)
}

pub fn gt(left: Expr, right: Expr) -> Expr {
    binary("gt", left, BinaryOp::Gt, right)
}

pub fn gteq(left: Expr, right: Expr) -> Expr {
    binary("gteq", left, BinaryOp::GtEq, right)
}

pub fn lt(left: Expr, right: Expr) -> Expr {
    binary("lt", left, BinaryOp::Lt, right)
}

pub fn lteq(left: Expr, right: Expr) -> Expr {
    binary("lteq", left, BinaryOp::LtEq, right)
}

pub fn and(left: Expr, right: Expr) -> Expr {
    binary("and", left, BinaryOp::And, right)
}

pub fn or(left: Expr, right: Expr) -> Expr {
    binary("or", left, BinaryOp::Or, right)
}

pub fn add(left: Expr, right: Expr) -> Expr {
    binary("add", left, BinaryOp::Add, right)
}

pub fn subtract(left: Expr, right: Expr) -> Expr {
    binary("subtract", left, BinaryOp::Subtract, right)
}

pub fn multiply(left: Expr, right: Expr) -> Expr {
    binary("multiply", left, BinaryOp::Multiply, right)
}

pub fn divide(left: Expr, right: Expr) -> Expr {
    binary("divide", left, BinaryOp::Divide, right)
}

pub fn modulus(left: Expr, right: Expr) -> Expr {
    binary("modulus", left, BinaryOp::Modulus, right)
}

impl<Rhs> std::ops::Add<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn add(self, rhs: Rhs) -> Self::Output {
        add(self, rhs.into())
    }
}

impl<Rhs> std::ops::Sub<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn sub(self, rhs: Rhs) -> Self::Output {
        subtract(self, rhs.into())
    }
}

impl<Rhs> std::ops::Mul<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn mul(self, rhs: Rhs) -> Self::Output {
        multiply(self, rhs.into())
    }
}

impl<Rhs> std::ops::Div<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn div(self, rhs: Rhs) -> Self::Output {
        divide(self, rhs.into())
    }
}

impl<Rhs> std::ops::Rem<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn rem(self, rhs: Rhs) -> Self::Output {
        modulus(self, rhs.into())
    }
}

// we can not overload '&&' or '||' in Rust so we use these instead
impl<Rhs> std::ops::BitAnd<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn bitand(self, rhs: Rhs) -> Self::Output {
        and(self, rhs.into())
    }
}

impl<Rhs> std::ops::BitOr<Rhs> for Expr
where
    Rhs: Into<Expr>,
{
    type Output = Expr;
    fn bitor(self, rhs: Rhs) -> Self::Output {
        or(self, rhs.into())
    }
}

#[derive(Clone)]
enum AggregateOp {
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
pub struct AggregateExpr {
    name: String,
    op: AggregateOp,
    expr: Expr,
}

impl AggregateExpr {
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

impl std::fmt::Display for AggregateExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.op, self.expr)
    }
}

fn aggregate(name: impl Into<String>, op: AggregateOp, expr: Expr) -> Expr {
    Expr::new(ExprKind::Aggregate(AggregateExpr {
        name: name.into(),
        op,
        expr,
    }))
}

pub fn min(expr: Expr) -> Expr {
    aggregate("min", AggregateOp::Min, expr)
}

pub fn max(expr: Expr) -> Expr {
    aggregate("max", AggregateOp::Max, expr)
}

pub fn avg(expr: Expr) -> Expr {
    aggregate("avg", AggregateOp::Avg, expr)
}

pub fn sum(expr: Expr) -> Expr {
    aggregate("sum", AggregateOp::Sum, expr)
}

pub fn count(expr: Expr) -> Expr {
    aggregate("count", AggregateOp::Count, expr)
}

#[derive(Clone)]
pub struct AliasExpr {
    expr: Expr,
    name: String,
}

impl AliasExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        let f = self.expr.to_field(input);
        Arc::new(Field::new(
            &self.name,
            f.data_type().clone(),
            f.is_nullable(),
        ))
    }
}

impl std::fmt::Display for AliasExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} AS {}", self.expr, self.name)
    }
}

pub fn alias(expr: Expr, name: impl Into<String>) -> Expr {
    Expr::new(ExprKind::Alias(AliasExpr {
        expr,
        name: name.into(),
    }))
}
