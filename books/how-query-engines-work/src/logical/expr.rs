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
pub struct Expr(Arc<ExprKind>);

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
enum ExprKind {
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

impl std::fmt::Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone)]
struct ColumnExpr {
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
struct LiteralExpr {
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
struct BinaryExpr {
    name: String,
    left: Expr,
    op: BinaryOp,
    right: Expr,
}

impl BinaryExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        let lf = self.left.to_field(input);
        let rf = self.right.to_field(input);

        let dtype = match self.op {
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Gt
            | BinaryOp::GtEq
            | BinaryOp::Lt
            | BinaryOp::LtEq
            | BinaryOp::And
            | BinaryOp::Or => DataType::Boolean,
            // TODO: coercion of datatype, right now only take left expr
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulus => lf.data_type().clone(),
        };

        Arc::new(Field::new(
            &self.name,
            dtype,
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

// TODO: all binary ops...

#[derive(Clone)]
enum AggregateOp {
    Min,
    Max,
    Avg,
    Count,
}

impl std::fmt::Display for AggregateOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::Avg => "AVG",
            Self::Count => "COUNT",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
struct AggregateExpr {
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

#[derive(Clone)]
struct AliasExpr {
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
