use std::sync::Arc;

use arrow::{
    array::{ArrayRef, RecordBatch},
    datatypes::{Schema, SchemaRef},
};

use crate::{data_source::RecordBatchIterator, physical::expr::PhysicalExpr};

#[derive(Clone)]
pub enum PhysicalPlanKind {
    Limit(PhysicalLimitPlan),
    Filter(PhysicalFilterPlan),
    Projection(PhysicalProjectionPlan),
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

    fn kind(&self) -> &PhysicalPlanKind {
        self.0.as_ref()
    }

    pub fn schema(&self) -> &SchemaRef {
        match self.kind() {
            PhysicalPlanKind::Limit(plan) => plan.schema(),
            PhysicalPlanKind::Projection(plan) => plan.schema(),
            PhysicalPlanKind::Filter(plan) => plan.schema(),
        }
    }

    pub fn inputs(&self) -> Vec<&PhysicalPlan> {
        match self.kind() {
            _ => todo!(),
        }
    }

    pub fn execute<'a>(&'a self, batches: RecordBatchIterator<'a>) -> RecordBatchIterator<'a> {
        match self.kind() {
            PhysicalPlanKind::Limit(plan) => plan.execute(batches),
            PhysicalPlanKind::Projection(plan) => plan.execute(batches),
            PhysicalPlanKind::Filter(plan) => plan.execute(batches),
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

impl std::fmt::Display for PhysicalPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            _ => write!(f, "TODO: implement this"),
        }
    }
}

#[derive(Clone)]
struct PhysicalLimitPlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    limit: usize,
}

impl PhysicalLimitPlan {
    // NOTE: this always goes through all batches even though we have already consumed
    // and sliced all batches of interest, ALSO, when we dont want any more rows we
    // produce empty record batches, stinky!
    //
    // Maybe it is just better to NOT use lazy iterators and materialize them at this stage,
    // since LIMIT should always be the last stage anyway... NO IT IS NOT, subqueries,
    // BUT WE DONT SUPPORT SUBQUERIES!
    //
    // NOTE: OK WE DO THE MATERIALIZATION! Maybe? investigate performance later!
    fn execute<'a>(&'a self, batches: RecordBatchIterator<'a>) -> RecordBatchIterator<'a> {
        // we can not do this! then we take `limit` from each batch!
        // Box::new(batches.into_iter().map(|b| b.slice(0, self.limit)))

        let mut remaining = self.limit;
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

        /*
        Box::new(batches.into_iter().map(move |b| {
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
        }))
        */
        Box::new(produced.into_iter())
    }

    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn input(&self) -> &PhysicalPlan {
        &self.input
    }
}

#[derive(Clone)]
struct PhysicalProjectionPlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    exprs: Vec<PhysicalExpr>,
}

impl PhysicalProjectionPlan {
    /*
    // FIND OUT SCHEMA OF ALL EXPRESSIONS AND COMBINE THEM
    fn new(input: PhysicalPlan, exprs: Vec<PhysicalExpr>) -> Self {
        let schema = Arc::new(Schema::from(
                exprs.iter().map(|expr| exprArc::new(Schema::from(
                        ));
            ));
        Self {
        }
    }
    */

    fn execute<'a>(&'a self, batches: RecordBatchIterator<'a>) -> RecordBatchIterator<'a> {
        // for each batch we want to evalute each expression on it and build a batch out of its results
        // ok, how xd,
        //
        // ok i think i got it
        Box::new(batches.into_iter().map(|b| {
            RecordBatch::try_new(
                self.schema.clone(),
                self.exprs
                    .iter()
                    .map(|expr| expr.evaluate(&b).to_arrow_array())
                    .collect::<Vec<ArrayRef>>(),
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

#[derive(Clone)]
struct PhysicalFilterPlan {
    input: PhysicalPlan,
    schema: SchemaRef,
    predicate: PhysicalExpr,
}

impl PhysicalFilterPlan {
    fn new(input: PhysicalPlan, predicate: PhysicalExpr) -> Self {
        // a filter operation has the same schema as its child
        let schema = input.schema().clone();
        Self {
            input,
            schema,
            predicate,
        }
    }

    fn execute<'a>(&'a self, batches: RecordBatchIterator<'a>) -> RecordBatchIterator<'a> {
        Box::new(batches.into_iter().map(|b| {
            RecordBatch::try_new(
                self.schema.clone(),
                vec![self.predicate.evaluate(&b).to_arrow_array()],
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
