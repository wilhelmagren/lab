use std::collections::hash_map::Entry;
use std::{collections::HashMap, sync::Arc};

use arrow::{
    array::{ArrayRef, AsArray, BooleanArray, RecordBatch, downcast_array},
    compute::{concat, filter_record_batch, min},
    datatypes::{DataType, FieldRef, Float64Type, Int8Type, Int64Type, Schema, SchemaRef},
    row::{OwnedRow, RowConverter, SortField},
};

use crate::{
    data_source::{DataSourceRef, RecordBatchIterator},
    logical::expr::AggregateOp,
    physical::expr::{ColumnarValue, PhysicalExpr, PhysicalExprKind},
    scalar::ScalarValue,
};

#[derive(Clone)]
pub enum PhysicalPlanKind {
    Limit(PhysicalLimitPlan),
    Scan(PhysicalScanPlan),
    Filter(PhysicalFilterPlan),
    Projection(PhysicalProjectionPlan),
    Aggregate(PhysicalAggregatePlan),
}

#[derive(Clone)]
pub struct PhysicalPlan(Arc<PhysicalPlanKind>);

type HashAgg = HashMap<OwnedRow, Vec<Accumulator>>;

//
// mental model:
//   a plan execution takes a sequence of record batches, and produces another sequence
//   of record batches, e.g., a ColumnPlan yields a sequence of RecordBatches only
//   containing one column, but another plan might yield rb's with more columns,
//   like a filter plan yields batches with the same schema as its input
//
//   a physical expr evaluates itself on ONE batch at a time and the physical plan
//   iterates the batch and applies evalute on all batches => yielding an iterator of batches
//
impl PhysicalPlan {
    pub fn new(plan: PhysicalPlanKind) -> Self {
        Self(Arc::new(plan))
    }

    pub fn kind(&self) -> &PhysicalPlanKind {
        self.0.as_ref()
    }

    pub fn schema(&self) -> &SchemaRef {
        match self.kind() {
            PhysicalPlanKind::Limit(plan) => plan.schema(),
            PhysicalPlanKind::Scan(plan) => plan.schema(),
            PhysicalPlanKind::Filter(plan) => plan.schema(),
            PhysicalPlanKind::Projection(plan) => plan.schema(),
            PhysicalPlanKind::Aggregate(plan) => plan.schema(),
        }
    }

    pub fn inputs(&self) -> Vec<&PhysicalPlan> {
        match self.kind() {
            PhysicalPlanKind::Limit(plan) => vec![plan.input()],
            PhysicalPlanKind::Scan(_) => vec![],
            PhysicalPlanKind::Filter(plan) => vec![plan.input()],
            PhysicalPlanKind::Projection(plan) => vec![plan.input()],
            PhysicalPlanKind::Aggregate(plan) => vec![plan.input()],
        }
    }

    pub fn execute(&self) -> RecordBatchIterator {
        match self.kind() {
            PhysicalPlanKind::Limit(plan) => plan.execute(),
            PhysicalPlanKind::Scan(plan) => plan.execute(),
            PhysicalPlanKind::Filter(plan) => plan.execute(),
            PhysicalPlanKind::Projection(plan) => plan.execute(),
            PhysicalPlanKind::Aggregate(plan) => plan.execute(),
        }
    }

    pub fn format(&self, indent: usize) -> String {
        let mut s = String::new();
        if indent > 0 {
            s.push_str(&"   ".repeat(indent - 1));
            s.push_str("+- ");
        }
        s.push_str(self.to_string().as_str());
        s.push_str("\n");
        self.inputs()
            .iter()
            .for_each(|i| s.push_str(i.format(indent + 1).as_str()));
        s
    }
}

impl std::fmt::Display for PhysicalPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            PhysicalPlanKind::Limit(plan) => plan.fmt(f),
            PhysicalPlanKind::Scan(plan) => plan.fmt(f),
            PhysicalPlanKind::Filter(plan) => plan.fmt(f),
            PhysicalPlanKind::Projection(plan) => plan.fmt(f),
            PhysicalPlanKind::Aggregate(plan) => plan.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct PhysicalLimitPlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    limit: usize,
}

impl PhysicalLimitPlan {
    pub fn new(input: PhysicalPlan, limit: usize) -> Self {
        let schema = input.schema().clone();
        Self {
            input,
            schema,
            limit,
        }
    }

    // NOTE: this always goes through all batches even though we have already consumed
    // and sliced all batches of interest, ALSO, when we dont want any more rows we
    // produce empty record batches, stinky!
    //
    // Maybe it is just better to NOT use lazy iterators and materialize them at this stage,
    // since LIMIT should always be the last stage anyway... NO IT IS NOT, subqueries,
    // BUT WE DONT SUPPORT SUBQUERIES!
    //
    // NOTE: OK WE DO THE MATERIALIZATION! Maybe? investigate performance later!
    // OK MAYBE NOT, WE CAN CREATE Empty batch and then filter on it?...
    fn execute(&self) -> RecordBatchIterator {
        // we can not do this! then we take `limit` from each batch!
        // Box::new(batches.into_iter().map(|b| b.slice(0, self.limit)))
        /*
        let mut produced = Vec::new();
        for batch in batches {
            let bnr = batch.num_rows();
            let to_take = remaining.min(bnr);
            if to_take == 0 {
                // empty batch? I DONT KNOW...
                continue;
            }
            // take the entire batch
            else if to_take >= bnr {
                remaining -= bnr;
                produced.push(batch);
            } else {
                remaining = 0;
                produced.push(batch.slice(0, to_take));
            }
        }
        Box::new(produced.into_iter())


        Box::new(
            batches
                .into_iter()
                .map(move |b| {
                    let bnr = b.num_rows();
                    let to_take = remaining.min(bnr);
                    if to_take == 0 {
                        // empty batch? I DONT KNOW...
                        return RecordBatch::new_empty(self.schema.clone());
                    }
                    // take the entire batch
                    else if to_take >= bnr {
                        remaining -= bnr;
                        return b;
                    } else {
                        remaining = 0;
                        return b.slice(0, to_take);
                    }
                })
                .filter(|b| b.num_rows() > 0),
        )
        */
        let mut batches = self.input.execute();
        let mut remaining = self.limit;
        Box::new(std::iter::from_fn(move || {
            if remaining == 0 {
                return None;
            }

            let batch = batches.next()?;
            let n_rows = batch.num_rows();

            if n_rows <= remaining {
                remaining -= n_rows;
                Some(batch)
            } else {
                let batch = batch.slice(0, remaining);
                remaining = 0;
                Some(batch)
            }
        }))
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn input(&self) -> &PhysicalPlan {
        &self.input
    }
}

impl From<PhysicalLimitPlan> for PhysicalPlan {
    fn from(value: PhysicalLimitPlan) -> Self {
        Self(Arc::new(PhysicalPlanKind::Limit(value)))
    }
}

impl std::fmt::Display for PhysicalLimitPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LimitExec: limit={}", self.limit)
    }
}

#[derive(Clone)]
pub struct PhysicalScanPlan {
    data_source: DataSourceRef,
    schema: SchemaRef,
    projection: Option<Vec<usize>>,
}

impl PhysicalScanPlan {
    pub fn new(data_source: DataSourceRef, projection: Option<&[usize]>) -> Self {
        let mut schema = data_source.schema().clone();
        if let Some(proj) = &projection {
            schema = Arc::new(Schema::new(
                data_source
                    .schema()
                    .clone()
                    .fields()
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| proj.contains(i))
                    .map(|(_, f)| f.clone() as FieldRef)
                    .collect::<Vec<_>>(),
            ));
            Self {
                data_source,
                schema,
                projection: Some(proj.to_vec()),
            }
        } else {
            Self {
                data_source,
                schema,
                projection: None,
            }
        }
    }

    fn projection(&self) -> Option<&[usize]> {
        if let Some(proj) = &self.projection {
            Some(proj)
        } else {
            None
        }
    }

    fn execute(&self) -> RecordBatchIterator {
        // THIS IS CLONE OF VEC, UH OH
        // i fuycking hate lifetimes
        self.data_source.scan(self.projection())
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }
}

impl From<PhysicalScanPlan> for PhysicalPlan {
    fn from(value: PhysicalScanPlan) -> Self {
        Self(Arc::new(PhysicalPlanKind::Scan(value)))
    }
}

fn format_schema(schema: &Schema) -> String {
    let mut s = String::new();
    s.push_str(
        &schema
            .fields()
            .iter()
            .map(|f| match f.is_nullable() {
                true => format!("{}({} nullable)", f.name(), f.data_type()),
                false => format!("{}({})", f.name(), f.data_type()),
            })
            .collect::<Vec<String>>()
            .join(", "),
    );
    s
}

impl std::fmt::Display for PhysicalScanPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScanExec: schema=[{}], projection={:?}",
            format_schema(&self.schema),
            self.projection
        )
    }
}

#[derive(Clone)]
pub struct PhysicalProjectionPlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    exprs: Vec<PhysicalExpr>,
}

impl PhysicalProjectionPlan {
    pub fn new(input: PhysicalPlan, exprs: Vec<PhysicalExpr>) -> Self {
        let schema = Arc::new(Schema::new(
            exprs
                .iter()
                .map(|expr| expr.to_field(input.schema()))
                .collect::<Vec<_>>(),
        ));

        Self {
            input,
            schema,
            exprs,
        }
    }

    fn execute(&self) -> RecordBatchIterator {
        let batches = self.input.execute();
        let schema = self.schema.clone();
        let exprs = self.exprs.clone();

        Box::new(batches.into_iter().map(move |b| {
            RecordBatch::try_from_iter(
                schema
                    .fields()
                    .iter()
                    .map(|f| f.name())
                    .zip(exprs.iter().map(|expr| expr.evaluate(&b).to_arrow_array())),
            )
            .unwrap()
        }))
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn input(&self) -> &PhysicalPlan {
        &self.input
    }
}

impl From<PhysicalProjectionPlan> for PhysicalPlan {
    fn from(value: PhysicalProjectionPlan) -> Self {
        Self(Arc::new(PhysicalPlanKind::Projection(value)))
    }
}

impl std::fmt::Display for PhysicalProjectionPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProjectionExec: {}",
            self.exprs
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone)]
pub struct PhysicalFilterPlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    predicate: PhysicalExpr,
}

impl PhysicalFilterPlan {
    pub fn new(input: PhysicalPlan, predicate: PhysicalExpr) -> Self {
        // a filter operation has the same schema as its child
        let schema = input.schema().clone();
        Self {
            input,
            schema,
            predicate,
        }
    }

    fn execute(&self) -> RecordBatchIterator {
        let batches = self.input.execute();
        let predicate = self.predicate.clone();

        Box::new(batches.map(move |batch| {
            let mask = predicate.evaluate(&batch).to_arrow_array();
            filter_record_batch(&batch, &downcast_array::<BooleanArray>(mask.as_ref())).unwrap()
        }))
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn input(&self) -> &PhysicalPlan {
        &self.input
    }

    pub fn predicate(&self) -> &PhysicalExpr {
        &self.predicate
    }
}

impl From<PhysicalFilterPlan> for PhysicalPlan {
    fn from(value: PhysicalFilterPlan) -> Self {
        Self(Arc::new(PhysicalPlanKind::Filter(value)))
    }
}

impl std::fmt::Display for PhysicalFilterPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FilterExec: predicate={}", self.predicate)
    }
}

#[derive(Clone, Debug)]
pub enum AccumulatorKind {
    Min(MinAccumulator),
    Max(MaxAccumulator),
    Sum(SumAccumulator),
    Avg(AvgAccumulator),
}

#[derive(Clone, Debug)]
pub struct Accumulator(AccumulatorKind);

impl Accumulator {
    pub fn kind(&mut self) -> &mut AccumulatorKind {
        &mut self.0
    }

    fn accumulate_row(&mut self, values: &ArrayRef, row: usize) {
        match self.kind() {
            AccumulatorKind::Min(acc) => acc.accumulate_row(values, row),
            AccumulatorKind::Max(acc) => acc.accumulate_row(values, row),
            AccumulatorKind::Sum(acc) => acc.accumulate_row(values, row),
            AccumulatorKind::Avg(acc) => acc.accumulate_row(values, row),
        }
    }

    /// merge intermediate states of two accumulators (self + other) into self
    fn merge(&mut self, other: &Accumulator) {
        match (self.kind(), &other.0) {
            (AccumulatorKind::Min(left), AccumulatorKind::Min(right)) => {
                left.merge(right);
            }
            (AccumulatorKind::Max(left), AccumulatorKind::Max(right)) => {
                left.merge(right);
            }
            (AccumulatorKind::Sum(left), AccumulatorKind::Sum(right)) => {
                left.merge(right);
            }
            (AccumulatorKind::Avg(left), AccumulatorKind::Avg(right)) => {
                left.merge(right);
            }
            _ => unreachable!("cannot merge different acc kinds"),
        }
    }

    fn final_value(&mut self) -> ScalarValue {
        match self.kind() {
            AccumulatorKind::Min(acc) => acc.final_value(),
            AccumulatorKind::Max(acc) => acc.final_value(),
            AccumulatorKind::Sum(acc) => acc.final_value(),
            AccumulatorKind::Avg(acc) => acc.final_value(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MaxAccumulator {
    pub value: ScalarValue,
}

impl From<MaxAccumulator> for Accumulator {
    fn from(value: MaxAccumulator) -> Self {
        Self(AccumulatorKind::Max(value))
    }
}
impl From<MinAccumulator> for Accumulator {
    fn from(value: MinAccumulator) -> Self {
        Self(AccumulatorKind::Min(value))
    }
}
impl From<SumAccumulator> for Accumulator {
    fn from(value: SumAccumulator) -> Self {
        Self(AccumulatorKind::Sum(value))
    }
}
impl From<AvgAccumulator> for Accumulator {
    fn from(value: AvgAccumulator) -> Self {
        Self(AccumulatorKind::Avg(value))
    }
}

impl MaxAccumulator {
    pub fn new() -> Self {
        Self {
            value: ScalarValue::Null,
        }
    }
    pub fn accumulate_row(&mut self, values: &ArrayRef, row: usize) {
        if values.is_null(row) {
            return;
        }

        match values.data_type() {
            DataType::Null => {}
            DataType::Boolean => {}
            DataType::Int8 => {
                let arr = values.as_primitive::<Int8Type>();
                let val = ScalarValue::Int8(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val > *current => self.value = val,
                    _ => {}
                }
            }
            DataType::Int16 => {}
            DataType::Int32 => {}
            DataType::Int64 => {
                let arr = values.as_primitive::<Int64Type>();
                let val = ScalarValue::Int64(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val > *current => self.value = val,
                    _ => {}
                }
            }
            DataType::UInt8 => {}
            DataType::UInt16 => {}
            DataType::UInt32 => {}
            DataType::UInt64 => {}
            DataType::Float32 => {}
            DataType::Float64 => {
                let arr = values.as_primitive::<Float64Type>();
                let val = ScalarValue::Float64(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val > *current => self.value = val,
                    _ => {}
                }
            }
            DataType::LargeUtf8 => {}
            _ => {}
        }
    }

    pub fn accumulate(&mut self, values: &ColumnarValue) {
        let xd = values.clone().to_arrow_array();
        let prim = match xd.data_type() {
            DataType::Int8 => xd.as_primitive::<Int8Type>(),
            _ => todo!(),
        };

        match self.value {
            ScalarValue::Null => {
                self.value = min(prim).into();
                return;
            }
            _ => {
                let xd = min(prim).into();
                if xd < self.value {
                    self.value = xd;
                }
                return;
            }
        }
    }

    pub fn final_value(&self) -> ScalarValue {
        self.value.clone()
    }

    /// Merge other intermediate values into this accumulator.
    pub fn merge(&mut self, other: &MaxAccumulator) {
        match &other.value {
            // noop
            ScalarValue::Null => {}
            value => match &self.value {
                ScalarValue::Null => {
                    self.value = value.clone();
                }
                current if value > current => {
                    self.value = value.clone();
                }
                _ => {}
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct MinAccumulator {
    pub value: ScalarValue,
}

impl MinAccumulator {
    pub fn new() -> Self {
        Self {
            value: ScalarValue::Null,
        }
    }
    pub fn accumulate_row(&mut self, values: &ArrayRef, row: usize) {
        if values.is_null(row) {
            return;
        }

        match values.data_type() {
            DataType::Null => {}
            DataType::Boolean => {}
            DataType::Int8 => {
                let arr = values.as_primitive::<Int8Type>();
                let val = ScalarValue::Int8(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val < *current => self.value = val,
                    _ => {}
                }
            }
            DataType::Int16 => {}
            DataType::Int32 => {}
            DataType::Int64 => {
                let arr = values.as_primitive::<Int64Type>();
                let val = ScalarValue::Int64(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val < *current => self.value = val,
                    _ => {}
                }
            }
            DataType::UInt8 => {}
            DataType::UInt16 => {}
            DataType::UInt32 => {}
            DataType::UInt64 => {}
            DataType::Float32 => {}
            DataType::Float64 => {
                let arr = values.as_primitive::<Float64Type>();
                let val = ScalarValue::Float64(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val < *current => self.value = val,
                    _ => {}
                }
            }
            DataType::LargeUtf8 => {}
            _ => {}
        }
    }

    pub fn accumulate(&mut self, values: &ColumnarValue) {
        let xd = values.clone().to_arrow_array();
        let prim = match xd.data_type() {
            DataType::Int8 => xd.as_primitive::<Int8Type>(),
            _ => todo!(),
        };

        match self.value {
            ScalarValue::Null => {
                self.value = min(prim).into();
                return;
            }
            _ => {
                let xd = min(prim).into();
                if xd < self.value {
                    self.value = xd;
                }
                return;
            }
        }
    }

    pub fn final_value(&self) -> ScalarValue {
        self.value.clone()
    }

    /// Merge other intermediate values into this accumulator.
    pub fn merge(&mut self, other: &MinAccumulator) {
        match &other.value {
            // noop
            ScalarValue::Null => {}
            value => match &self.value {
                ScalarValue::Null => {
                    self.value = value.clone();
                }
                current if value < current => {
                    self.value = value.clone();
                }
                _ => {}
            },
        }
    }

    fn update(&mut self, value: &ScalarValue) {
        match &self.value {
            ScalarValue::Null => {
                self.value = value.clone();
            }
            current if value < current => {
                self.value = value.clone();
            }
            _ => {}
        }
    }
}

/// value is accumulated sum and count is number of rows for the group
#[derive(Clone, Debug)]
pub struct AvgAccumulator {
    pub value: ScalarValue,
    pub count: usize,
}

impl AvgAccumulator {
    pub fn new() -> Self {
        Self {
            value: ScalarValue::Null,
            count: 0,
        }
    }
    pub fn accumulate_row(&mut self, values: &ArrayRef, row: usize) {
        self.count += 1;
        if values.is_null(row) {
            return;
        }

        match values.data_type() {
            DataType::Null => {}
            DataType::Boolean => {}
            DataType::Int8 => {
                let arr = values.as_primitive::<Int8Type>();
                let val = ScalarValue::Int8(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val < *current => self.value = val,
                    _ => {}
                }
            }
            DataType::Int16 => {}
            DataType::Int32 => {}
            DataType::Int64 => {
                let arr = values.as_primitive::<Int64Type>();
                let val = arr.value(row);
                match &mut self.value {
                    ScalarValue::Null => {
                        self.value = ScalarValue::Int64(val);
                    }
                    ScalarValue::Int64(curr) => {
                        *curr += val;
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            DataType::UInt8 => {}
            DataType::UInt16 => {}
            DataType::UInt32 => {}
            DataType::UInt64 => {}
            DataType::Float32 => {}
            DataType::Float64 => {
                let arr = values.as_primitive::<Float64Type>();
                let val = arr.value(row);
                match &mut self.value {
                    ScalarValue::Null => {
                        self.value = ScalarValue::Float64(val);
                    }
                    ScalarValue::Float64(curr) => {
                        *curr += val;
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            DataType::LargeUtf8 => {}
            _ => {}
        }
    }

    pub fn accumulate(&mut self, values: &ColumnarValue) {
        let xd = values.clone().to_arrow_array();
        let prim = match xd.data_type() {
            DataType::Int8 => xd.as_primitive::<Int8Type>(),
            _ => todo!(),
        };

        match self.value {
            ScalarValue::Null => {
                self.value = min(prim).into();
                return;
            }
            _ => {
                let xd = min(prim).into();
                if xd < self.value {
                    self.value = xd;
                }
                return;
            }
        }
    }

    pub fn final_value(&self) -> ScalarValue {
        match self.value {
            ScalarValue::Null => todo!(),
            ScalarValue::Boolean(_) => todo!(),
            ScalarValue::Int8(_) => todo!(),
            ScalarValue::Int16(_) => todo!(),
            ScalarValue::Int32(_) => todo!(),
            ScalarValue::Int64(_) => todo!(),
            ScalarValue::UInt8(_) => todo!(),
            ScalarValue::UInt16(_) => todo!(),
            ScalarValue::UInt32(_) => todo!(),
            ScalarValue::UInt64(_) => todo!(),
            ScalarValue::Float32(_) => todo!(),
            ScalarValue::Float64(sum) => (sum / self.count as f64).into(),
            ScalarValue::Utf8(_) => todo!(),
        }
    }

    /// Merge other intermediate values into this accumulator.
    pub fn merge(&mut self, other: &AvgAccumulator) {
        match (&mut self.value, &other.value) {
            (_, ScalarValue::Null) => {}
            (ScalarValue::Null, value) => {
                self.value = value.clone();
                self.count = other.count;
            }
            (ScalarValue::Int64(a), ScalarValue::Int64(b)) => {
                *a += *b;
                self.count += other.count;
            }
            (ScalarValue::Float64(a), ScalarValue::Float64(b)) => {
                // self.value = ScalarValue::Float64(*a + *b);
                *a += *b;
                self.count += other.count;
            }
            // i only support f64 right now
            _ => todo!(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct SumAccumulator {
    pub value: ScalarValue,
}

impl SumAccumulator {
    pub fn new() -> Self {
        Self {
            value: ScalarValue::Null,
        }
    }

    pub fn accumulate_row(&mut self, values: &ArrayRef, row: usize) {
        if values.is_null(row) {
            return;
        }

        match values.data_type() {
            DataType::Null => {}
            DataType::Boolean => {}
            DataType::Int8 => {
                let arr = values.as_primitive::<Int8Type>();
                let val = ScalarValue::Int8(arr.value(row));
                match &self.value {
                    ScalarValue::Null => {
                        self.value = val;
                    }
                    current if val < *current => self.value = val,
                    _ => {}
                }
            }
            DataType::Int16 => {}
            DataType::Int32 => {}
            DataType::Int64 => {
                let arr = values.as_primitive::<Int64Type>();
                let val = arr.value(row);
                match &mut self.value {
                    ScalarValue::Null => {
                        self.value = ScalarValue::Int64(val);
                    }
                    ScalarValue::Int64(curr) => {
                        *curr += val;
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            DataType::UInt8 => {}
            DataType::UInt16 => {}
            DataType::UInt32 => {}
            DataType::UInt64 => {}
            DataType::Float32 => {}
            DataType::Float64 => {
                let arr = values.as_primitive::<Float64Type>();
                let val = arr.value(row);
                match &mut self.value {
                    ScalarValue::Null => {
                        self.value = ScalarValue::Float64(val);
                    }
                    ScalarValue::Float64(curr) => {
                        *curr += val;
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
            DataType::LargeUtf8 => {}
            _ => {}
        }
    }

    pub fn accumulate(&mut self, values: &ColumnarValue) {
        let xd = values.clone().to_arrow_array();
        let prim = match xd.data_type() {
            DataType::Int8 => xd.as_primitive::<Int8Type>(),
            _ => todo!(),
        };

        match self.value {
            ScalarValue::Null => {
                self.value = min(prim).into();
                return;
            }
            _ => {
                let xd = min(prim).into();
                if xd < self.value {
                    self.value = xd;
                }
                return;
            }
        }
    }

    pub fn final_value(&self) -> ScalarValue {
        self.value.clone()
    }

    /// Merge other intermediate values into this accumulator.
    pub fn merge(&mut self, other: &SumAccumulator) {
        match (&mut self.value, &other.value) {
            (_, ScalarValue::Null) => {}
            (ScalarValue::Null, value) => {
                self.value = value.clone();
            }
            (ScalarValue::Int64(a), ScalarValue::Int64(b)) => {
                *a += *b;
            }
            (ScalarValue::Float64(a), ScalarValue::Float64(b)) => {
                // self.value = ScalarValue::Float64(*a + *b);
                *a += *b;
            }
            // i only support f64 right now
            _ => todo!(),
        }
    }

    fn update(&mut self, value: ScalarValue) {
        todo!()
    }
}

fn make_accumulators(ops: &[AggregateOp]) -> Vec<Accumulator> {
    ops.iter()
        .map(|op| match op {
            AggregateOp::Min => MinAccumulator::new().into(),
            AggregateOp::Max => MaxAccumulator::new().into(),
            AggregateOp::Sum => SumAccumulator::new().into(),
            AggregateOp::Avg => AvgAccumulator::new().into(),
            _ => unreachable!(),
        })
        .collect::<Vec<_>>()
}

fn merge_hashaggs(mut left: HashAgg, right: HashAgg) -> HashAgg {
    for (key, right_accs) in right {
        match left.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(right_accs);
            }
            Entry::Occupied(mut entry) => {
                let leftaccs = entry.get_mut();
                debug_assert_eq!(leftaccs.len(), right_accs.len());
                for (lacc, racc) in leftaccs.iter_mut().zip(right_accs.iter()) {
                    lacc.merge(racc);
                }
            }
        }
    }
    left
}

#[derive(Clone)]
pub struct PhysicalAggregatePlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    group_exprs: Vec<PhysicalExpr>,
    agg_exprs: Vec<PhysicalExpr>,
}

impl PhysicalAggregatePlan {
    pub fn new(
        input: PhysicalPlan,
        group_exprs: Vec<PhysicalExpr>,
        agg_exprs: Vec<PhysicalExpr>,
    ) -> Self {
        let input_schema = input.schema().clone();
        let schema = Arc::new(Schema::new(
            group_exprs
                .iter()
                .map(|expr| expr.to_field(&input_schema))
                .chain(agg_exprs.iter().map(|expr| expr.to_field(&input_schema)))
                .collect::<Vec<_>>(),
        ));

        Self {
            input,
            schema,
            group_exprs,
            agg_exprs,
        }
    }

    /// Assume we have two batches like
    ///   name, runes
    /// [
    ///   ["wilhelm", 1234],
    ///   ["elin", 124910],
    ///   ["elin", -4198],
    ///   ["wilhelm", 14],
    ///   ["leffe", 1337],
    /// ]
    ///
    /// and
    ///
    /// [
    ///   ["wilhelm", -2],
    ///   ["leffe", 49819],
    /// ]
    ///
    /// SELECT name, min(runes) FROM batch
    /// GROUP BY name
    ///
    /// What we want to achieve after first batch is:
    ///  { "wilhelm": Acc(14), "elin": Acc(-4198), "leffe": Acc(1337) }
    ///
    /// and after second
    ///  { "wilhelm": Acc(-2), "elin": Acc(-4198), "leffe": Acc(1337) }
    ///
    /// How do we get here? we need to hash on all evaluated groupBy expressions
    /// when I do expr.evalute(batch) i get column array of ["wilhelm", "elin", "elin", "wilhelm", "leffe"]
    /// we want to get the unique keys, for each unique value in that column we want to construct an accumulator
    /// and give the agg_expr.evalute(batch) result to the accumulator,
    /// if we have N groupBy cols, and M aggExpressions, we have to for each groupBy col and for each aggExpression
    /// accumulate(agg_expr.evaluate(batch))
    ///
    // THIS IS BLOCKING
    pub fn execute(&self) -> RecordBatchIterator {
        let batches = self.input.execute();

        // this is a wide transformation, meaning, we need to materialize
        // the full input before we can continue from this operation

        // we need to hash by the group_exprs, and for that we need to... evaluate the exprs?..
        // and transpose to row format

        let sort_fields = self
            .group_exprs
            .iter()
            .map(|e| SortField::new(e.to_field(self.input.schema()).data_type().clone()))
            .collect();

        let row_converter = RowConverter::new(sort_fields).unwrap();
        let mut hashagg = HashAgg::new();

        /*
        let agg_ops = self
            .agg_exprs
            .iter()
            .map(|e| match e.kind() {
                PhysicalExprKind::Aggregate(agg) => agg.op.clone(),
                _ => unreachable!("expected agg expression"),
            })
            .collect::<Vec<_>>();
        */

        // O(k) where k is the number of batches
        for batch in batches {
            // this gives us [["a","b","a","c","b"], [1,2,3,1,1]]
            let columnar_group_keys = self
                .group_exprs
                .iter()
                .map(|e| e.evaluate(&batch).to_arrow_array())
                .collect::<Vec<ArrayRef>>();

            // here we have [["a", 1],["b", 2],["a", 3],["c", 1],["b", 1]]
            // which are the unique groups that we want to partition on? we just put them in the hashmap and they become unique, problem solved
            let group_keys = row_converter.convert_columns(&columnar_group_keys).unwrap();

            // Aggregate input expressions, evaluated ONCE per batch
            let agg_values = self
                .agg_exprs
                .iter()
                .map(|e| match e.kind() {
                    PhysicalExprKind::Aggregate(agg) => agg.expr.evaluate(&batch).to_arrow_array(),
                    _ => unreachable!("AggregateExec must contain agg expressions only"),
                })
                .collect::<Vec<_>>();

            /*
            let batch_hashagg = (0..batch.num_rows())
                .into_par_iter()
                .fold(HashAgg::new, |mut local_map, row_idx| {
                    let gk = group_keys.row(row_idx).owned();
                    let accs = local_map
                        .entry(gk)
                        .or_insert_with(|| make_accumulators(&agg_ops));
                    for (acc, values) in accs.iter_mut().zip(agg_values.iter()) {
                        acc.accumulate_row(values, row_idx);
                    }
                    local_map
                })
                .reduce(HashAgg::new, merge_hashaggs);

            hashagg = merge_hashaggs(hashagg, batch_hashagg)
            */

            // O(n) where n is the number of rows in the batch
            for (row_idx, group_key) in group_keys.iter().enumerate() {
                // if the groupkey entry does not exist, create a new accumulator for each agg expression
                let accumulators = hashagg.entry(group_key.owned()).or_insert_with(|| {
                    self.agg_exprs
                        .iter()
                        .map(|e| {
                            let agg = match e.kind() {
                                PhysicalExprKind::Aggregate(agg) => agg,
                                _ => unreachable!(),
                            };
                            let acc = match agg.op {
                                AggregateOp::Min => MinAccumulator::new().into(),
                                AggregateOp::Max => MaxAccumulator::new().into(),
                                AggregateOp::Sum => SumAccumulator::new().into(),
                                AggregateOp::Avg => AvgAccumulator::new().into(),
                                _ => unreachable!(),
                            };
                            // dbg!(&agg.op);
                            // dbg!(&acc);
                            acc
                        })
                        .collect::<Vec<Accumulator>>()
                });

                // ~O(m) where m is the number of aggregate expressions
                for (acc, values) in accumulators.iter_mut().zip(agg_values.iter()) {
                    acc.accumulate_row(values, row_idx);
                }
            }
        }

        // TODO: Avg is wrong because for each accumulator we calculate the avg
        // and then what do we do with that value?

        // ("a", 1) -> [min=3, count=7]
        // ("b", 2) -> [min=8, count=2]
        // ("c", 1) -> [min=4, count=5]
        //
        // build two sets of output columns
        // group cols  agg cols
        // ----------  --------
        //  "a", 1      3, 7
        //  "b", 2      8, 2
        //  "c", 1      4, 5

        // this becomes [["a", 1], ["b", 2], ["c", 1]]
        let mut group_rows = Vec::new();

        // this is [[Array(3), Array(7)], [Array(8), Array(2)], [Array(4), Array(5)]]
        let mut agg_values = Vec::new();
        for (gk, accs) in hashagg.iter_mut() {
            group_rows.push(gk);
            agg_values.push(
                accs.iter_mut()
                    .map(|acc| acc.final_value().clone().to_arrow_scalar().into_inner())
                    .collect::<Vec<ArrayRef>>(),
            );
        }

        let group_cols = row_converter
            .convert_rows(group_rows.iter().map(|r| r.row()))
            .unwrap();
        let mut agg_cols = Vec::new();
        for agg_idx in 0..self.agg_exprs.len() {
            let arrs = agg_values
                .iter()
                .map(|gv| gv[agg_idx].as_ref())
                .collect::<Vec<_>>();
            agg_cols.push(concat(&arrs).unwrap());
        }

        let batch = RecordBatch::try_new(
            self.schema.clone(),
            group_cols
                .iter()
                .chain(agg_cols.iter())
                .map(|a| a.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();
        Box::new(std::iter::once(batch))
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn input(&self) -> &PhysicalPlan {
        &self.input
    }
}

impl From<PhysicalAggregatePlan> for PhysicalPlan {
    fn from(value: PhysicalAggregatePlan) -> Self {
        Self(Arc::new(PhysicalPlanKind::Aggregate(value)))
    }
}

impl std::fmt::Display for PhysicalAggregatePlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HashAggregateExec: groupBy=[{}], aggExpr=[{}]",
            self.group_exprs
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.agg_exprs
                .iter()
                .map(|expr| expr.to_string())
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}
