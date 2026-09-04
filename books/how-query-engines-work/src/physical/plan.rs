use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use arrow::{
    array::{ArrayRef, AsArray, BooleanArray, Int8Array, RecordBatch, downcast_array},
    compute::{filter_record_batch, min},
    datatypes::{DataType, FieldRef, Int8Type, Schema, SchemaRef},
    row::{RowConverter, SortField},
};

use crate::{
    data_source::{DataSourceRef, RecordBatchIterator},
    physical::expr::{ColumnarValue, PhysicalExpr},
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

impl std::fmt::Display for PhysicalScanPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScanExec: schema={}, projection={:?}",
            self.schema, self.projection
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

#[derive(Clone)]
pub enum AccumulatorKind {
    Min(MinAccumulator),
}

#[derive(Clone)]
pub struct MinAccumulator {
    value: ScalarValue,
}

impl MinAccumulator {
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

    pub fn final_value(&self) -> &ScalarValue {
        &self.value
    }

    /// Merge other intermediate values into this accumulator.
    pub fn merge(&mut self, other: &ColumnarValue) {
        self.accumulate(other);
    }
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
    pub fn execute(&self) -> RecordBatchIterator {
        let batches = self.input.execute();

        // this is a wide transformation, meaning, we need to materialize
        // the full input before we can continue from this operation

        // we need to hash by the group_exprs, and for that we need to... evaluate the exprs?..
        // and tranpose to row format

        let sort_fields = self
            .group_exprs
            .iter()
            .map(|e| SortField::new(e.to_field(self.input.schema()).data_type().clone()))
            .collect();
        let mut row_converter = RowConverter::new(sort_fields).unwrap();

        let mut hashagg: HashMap<String, Vec<MinAccumulator>> = HashMap::new();
        for batch in batches {
            // for simplicity, assume we only have one groupBy col
            // this gives us [["a","b","a","c","b"]], a columnar array with ALL group keys
            let all_group_keys = self
                .group_exprs
                .iter()
                .map(|e| e.evaluate(&batch).to_arrow_array())
                .collect::<Vec<ArrayRef>>();
            let rows = row_converter.convert_columns(&all_group_keys).unwrap();


            // for each group expression, for each group key, for each agg expr

            for group_keys in all_group_keys {
                // i give up
            }

            for gk in group_keys {
                if let Some(accs) = hashagg.get(&format!("{:?}", gk.unwrap())) {
                    for expr in self.agg_exprs {
                        for acc in accs.iter_mut() {
                            acc.accumulate(&expr.evaluate(&batch))
                        }
                    }
                }
            }
        }

        todo!()
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
            "AggregateExec: groupBy={}, aggExpr={}",
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
