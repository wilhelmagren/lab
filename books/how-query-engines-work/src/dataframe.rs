use std::sync::Arc;

use crate::{
    data_source::{ParquetDataSource, ScanProjection},
    logical_plan::{
        Aggregate, Filter, Join, JoinType, LogicalExprRef, LogicalPlanRef, Projection, Scan,
    },
    schema::SchemaRef,
};

pub struct DataFrame {
    plan: LogicalPlanRef,
}

impl DataFrame {
    /// Return the current logical plan of the dataframe.
    pub fn logical_plan(&self) -> LogicalPlanRef {
        self.plan.clone()
    }

    pub fn print_logical_plan(&self) {
        println!("{}", self.plan.format(0));
    }

    /// Project columns on the dataframe.
    pub fn project(&self, exprs: Vec<LogicalExprRef>) -> Self {
        Self {
            plan: Arc::new(Projection::new(self.plan.clone(), exprs)),
        }
    }

    /// Filter the dataframe based on the provided expression.
    pub fn filter(&self, predicate: LogicalExprRef) -> Self {
        Self {
            plan: Arc::new(Filter::new(self.plan.clone(), predicate)),
        }
    }

    /// Aggregate columns on the dataframe.
    pub fn agg(&self, group_by: Vec<LogicalExprRef>, aggregations: Vec<LogicalExprRef>) -> Self {
        Self {
            plan: Arc::new(Aggregate::new(self.plan.clone(), group_by, aggregations)),
        }
    }

    /// Join two dataframes together with the specified method.
    pub fn join(&self, right: DataFrame, how: JoinType, on: Vec<(String, String)>) -> Self {
        Self {
            plan: Arc::new(Join::new(self.plan.clone(), right.logical_plan(), how, on)),
        }
    }

    /// Return the schema of the dataframe.
    pub fn schema(&self) -> SchemaRef {
        self.plan.schema().clone()
    }
}

pub struct ExecutionContext;

impl ExecutionContext {
    pub fn parquet(filename: impl Into<String>, projection: Option<ScanProjection>) -> DataFrame {
        DataFrame {
            plan: Arc::new(Scan::new(
                Arc::new(ParquetDataSource::new(filename.into())),
                projection,
            )),
        }
    }
}
