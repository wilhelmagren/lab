use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Datum, RecordBatch, Scalar, downcast_array};
use arrow::compute::kernels::boolean::{and, or};
use arrow::compute::kernels::cmp::{eq, gt, gt_eq, lt, lt_eq, neq};
use arrow::compute::kernels::numeric::{add, div, mul, rem, sub};
use arrow::datatypes::{Field, FieldRef, Schema};
use arrow::util::display::array_value_to_string;

use crate::logical::expr::{AggregateOp, BinaryOp};

#[derive(Clone, Debug)]
pub enum PhysicalExprKind {
    Column(PhysicalColumnExpr),
    Literal(PhysicalLiteralExpr),
    Binary(PhysicalBinaryExpr),
    Aggregate(PhysicalAggregateExpr),
}

#[derive(Clone, Debug)]
pub struct PhysicalExpr(Arc<PhysicalExprKind>);

impl PhysicalExpr {
    pub fn new(kind: PhysicalExprKind) -> Self {
        Self(Arc::new(kind))
    }

    pub fn kind(&self) -> &PhysicalExprKind {
        self.0.as_ref()
    }

    pub fn to_field(&self, input: &Schema) -> FieldRef {
        match self.kind() {
            PhysicalExprKind::Column(expr) => expr.to_field(input),
            PhysicalExprKind::Literal(expr) => expr.to_field(),
            PhysicalExprKind::Binary(expr) => expr.to_field(input),
            PhysicalExprKind::Aggregate(expr) => {
                let f = expr.expr.to_field(input);
                Arc::new(f.as_ref().clone().with_name(expr.name.clone()))
            }
        }
    }

    pub fn evaluate(&self, input: &RecordBatch) -> ColumnarValue {
        match self.kind() {
            PhysicalExprKind::Column(expr) => expr.evaluate(&input),
            PhysicalExprKind::Literal(expr) => expr.evaluate(),
            PhysicalExprKind::Binary(expr) => expr.evaluate(input),
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for PhysicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            PhysicalExprKind::Column(expr) => expr.fmt(f),
            PhysicalExprKind::Literal(expr) => expr.fmt(f),
            PhysicalExprKind::Binary(expr) => expr.fmt(f),
            PhysicalExprKind::Aggregate(expr) => expr.fmt(f),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ColumnarValue {
    Array(ArrayRef),
    Scalar(Scalar<ArrayRef>),
}

impl ColumnarValue {
    pub(crate) fn to_arrow_array(self) -> ArrayRef {
        match self {
            Self::Array(arr) => arr,
            Self::Scalar(s) => s.into_inner(),
        }
    }

    pub(crate) fn as_datum(&self) -> &dyn Datum {
        match self {
            Self::Array(arr) => arr,
            Self::Scalar(scalar) => scalar,
        }
    }
}

/// Reference a column (field) in a batch by index.
#[derive(Clone, Debug)]
pub struct PhysicalColumnExpr {
    index: usize,
}

impl PhysicalColumnExpr {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    fn to_field(&self, input: &Schema) -> FieldRef {
        Arc::new(input.field(self.index).clone())
    }

    fn evaluate(&self, batch: &RecordBatch) -> ColumnarValue {
        ColumnarValue::Array(batch.column(self.index).clone())
    }
}

impl std::fmt::Display for PhysicalColumnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.index)
    }
}

impl From<PhysicalColumnExpr> for PhysicalExpr {
    fn from(value: PhysicalColumnExpr) -> Self {
        Self(Arc::new(PhysicalExprKind::Column(value)))
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalLiteralExpr {
    value: Scalar<ArrayRef>,
}

impl PhysicalLiteralExpr {
    pub fn new(value: Scalar<ArrayRef>) -> Self {
        Self { value }
    }

    fn to_field(&self) -> FieldRef {
        Arc::new(Field::new(
            // MAYBE THIS IS EXPENSIVE IDK
            self.to_string(),
            self.value.clone().into_inner().data_type().clone(),
            false,
        ))
    }

    fn evaluate(&self) -> ColumnarValue {
        ColumnarValue::Scalar(self.value.clone())
    }
}

impl std::fmt::Display for PhysicalLiteralExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (arr, _) = self.value.get();
        let value = array_value_to_string(arr, 0).unwrap();
        write!(f, "{}", value)
    }
}

impl From<PhysicalLiteralExpr> for PhysicalExpr {
    fn from(value: PhysicalLiteralExpr) -> Self {
        Self(Arc::new(PhysicalExprKind::Literal(value)))
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalBinaryExpr {
    name: String,
    left: PhysicalExpr,
    op: BinaryOp,
    right: PhysicalExpr,
}

impl PhysicalBinaryExpr {
    pub fn new(
        name: impl Into<String>,
        left: PhysicalExpr,
        op: BinaryOp,
        right: PhysicalExpr,
    ) -> Self {
        Self {
            name: name.into(),
            left,
            op,
            right,
        }
    }

    fn to_field(&self, input: &Schema) -> FieldRef {
        let lf = self.left.to_field(input);
        let rf = self.right.to_field(input);

        Arc::new(Field::new(
            &self.name,
            self.op.result_data_type(lf.data_type(), rf.data_type()),
            lf.is_nullable() || rf.is_nullable(),
        ))
    }

    // can a binary expression return a scalar value? dont think so
    fn evaluate(&self, input: &RecordBatch) -> ColumnarValue {
        let lc = self.left.evaluate(input);
        let rc = self.right.evaluate(input);

        let arr = match self.op {
            BinaryOp::Add => add(lc.as_datum(), rc.as_datum()).unwrap(),
            BinaryOp::Subtract => sub(lc.as_datum(), rc.as_datum()).unwrap(),
            BinaryOp::Multiply => mul(lc.as_datum(), rc.as_datum()).unwrap(),
            BinaryOp::Divide => div(lc.as_datum(), rc.as_datum()).unwrap(),
            BinaryOp::Modulus => rem(lc.as_datum(), rc.as_datum()).unwrap(),
            BinaryOp::Eq => Arc::new(eq(lc.as_datum(), rc.as_datum()).unwrap()),
            BinaryOp::NotEq => Arc::new(neq(lc.as_datum(), rc.as_datum()).unwrap()),
            BinaryOp::Gt => Arc::new(gt(lc.as_datum(), rc.as_datum()).unwrap()),
            BinaryOp::GtEq => Arc::new(gt_eq(lc.as_datum(), rc.as_datum()).unwrap()),
            BinaryOp::Lt => Arc::new(lt(lc.as_datum(), rc.as_datum()).unwrap()),
            BinaryOp::LtEq => Arc::new(lt_eq(lc.as_datum(), rc.as_datum()).unwrap()),
            // how do we handle this!!! downcast_ref ? maybe ;)
            // YES WE DO!
            // SAFETY: PANICS IF LEFT and RIGHT IS NOT BOOLEAN ARRAY!
            BinaryOp::And => Arc::new(
                and(
                    &downcast_array::<BooleanArray>(&lc.to_arrow_array()),
                    &downcast_array::<BooleanArray>(&rc.to_arrow_array()),
                )
                .unwrap(),
            ),
            BinaryOp::Or => Arc::new(
                or(
                    &downcast_array::<BooleanArray>(&lc.to_arrow_array()),
                    &downcast_array::<BooleanArray>(&rc.to_arrow_array()),
                )
                .unwrap(),
            ),
        };

        ColumnarValue::Array(arr)
    }
}

impl std::fmt::Display for PhysicalBinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.op, self.right)
    }
}

impl From<PhysicalBinaryExpr> for PhysicalExpr {
    fn from(value: PhysicalBinaryExpr) -> Self {
        Self(Arc::new(PhysicalExprKind::Binary(value)))
    }
}

#[derive(Clone, Debug)]
pub struct PhysicalAggregateExpr {
    pub op: AggregateOp,
    pub expr: PhysicalExpr,
    pub name: String,
}

impl PhysicalAggregateExpr {
    fn to_field(&self, input: &Schema) -> FieldRef {
        self.expr.to_field(input)
    }
}

impl std::fmt::Display for PhysicalAggregateExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.op, self.expr)
    }
}
