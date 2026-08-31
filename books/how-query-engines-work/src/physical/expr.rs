use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, RecordBatch, Scalar, downcast_array};
use arrow::compute::kernels::aggregate::{max, min, sum};
use arrow::compute::kernels::boolean::{and, or};
use arrow::compute::kernels::cmp::{eq, gt, gt_eq, lt, lt_eq, neq};
use arrow::compute::kernels::numeric::{add, div, mul, rem, sub};
use arrow::datatypes::SchemaRef;

use crate::logical::expr::{AggregateOp, BinaryOp};
use crate::scalar::ScalarValue;

#[derive(Clone)]
pub enum PhysicalExprKind {
    Column(PhysicalColumnExpr),
}

#[derive(Clone)]
pub struct PhysicalExpr(Arc<PhysicalExprKind>);

impl PhysicalExpr {
    pub fn new(kind: PhysicalExprKind) -> Self {
        Self(Arc::new(kind))
    }

    pub fn kind(&self) -> &PhysicalExprKind {
        self.0.as_ref()
    }

    pub fn evaluate(&self, input: &RecordBatch) -> ColumnarValue {
        match self.kind() {
            PhysicalExprKind::Column(expr) => expr.evaluate(&input),
        }
    }
}

impl std::fmt::Display for PhysicalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TODO")
    }
}

#[derive(Clone)]
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

    pub(crate) fn to_record_batch(self, schema: &SchemaRef) -> RecordBatch {
        RecordBatch::try_new(schema.clone(), vec![self.to_arrow_array()]).unwrap()
    }
}

/// Reference a column (field) in a batch by index.
#[derive(Clone)]
struct PhysicalColumnExpr {
    index: usize,
}

impl PhysicalColumnExpr {
    fn evaluate(&self, batch: &RecordBatch) -> ColumnarValue {
        ColumnarValue::Array(batch.column(self.index).clone())
    }
}

#[derive(Clone)]
struct PhysicalLiteralExpr {
    value: Scalar<ArrayRef>,
}

impl PhysicalLiteralExpr {
    fn evaluate(&self, _input: &RecordBatch) -> ColumnarValue {
        ColumnarValue::Scalar(self.value.clone())
    }
}

#[derive(Clone)]
struct PhysicalBinaryExpr {
    left: PhysicalExpr,
    op: BinaryOp,
    right: PhysicalExpr,
}

impl PhysicalBinaryExpr {
    // can a binary expression return a scalar value? dont think so
    fn evaluate(&self, input: &RecordBatch) -> ColumnarValue {
        let larr = &self.left.evaluate(input).to_arrow_array();
        let rarr = &self.right.evaluate(input).to_arrow_array();

        let arr = match self.op {
            BinaryOp::Add => add(larr, rarr).unwrap(),
            BinaryOp::Subtract => sub(larr, rarr).unwrap(),
            BinaryOp::Multiply => mul(larr, rarr).unwrap(),
            BinaryOp::Divide => div(larr, rarr).unwrap(),
            BinaryOp::Modulus => rem(larr, rarr).unwrap(),
            BinaryOp::Eq => Arc::new(eq(larr, rarr).unwrap()),
            BinaryOp::NotEq => Arc::new(neq(larr, rarr).unwrap()),
            BinaryOp::Gt => Arc::new(gt(larr, rarr).unwrap()),
            BinaryOp::GtEq => Arc::new(gt_eq(larr, rarr).unwrap()),
            BinaryOp::Lt => Arc::new(lt(larr, rarr).unwrap()),
            BinaryOp::LtEq => Arc::new(lt_eq(larr, rarr).unwrap()),
            // how do we handle this!!! downcast_ref ? maybe ;)
            // YES WE DO!
            // SAFETY: PANICS IF LEFT and RIGHT IS NOT BOOLEAN ARRAY!
            BinaryOp::And => Arc::new(
                and(
                    &downcast_array::<BooleanArray>(larr),
                    &downcast_array::<BooleanArray>(rarr),
                )
                .unwrap(),
            ),
            BinaryOp::Or => Arc::new(
                or(
                    &downcast_array::<BooleanArray>(larr),
                    &downcast_array::<BooleanArray>(rarr),
                )
                .unwrap(),
            ),
        };

        ColumnarValue::Array(arr)
    }
}

/*
// HOW TO DO AGGREGATES? THEY NEED STATE ACROSS BATCHES.
#[derive(Clone)]
struct PhysicalAggregateExpr {
    op: AggregateOp,
    expr: PhysicalExpr,
    accumulated: Option<ScalarValue>,
}

pub enum Accumulator {
    Min(MinAccumulator),
    Max(MaxAccumulator),
    Avg(AvgAccumulator),
    Sum(SumAccumulator),
    Count(CountAccumulator),
}
*/
