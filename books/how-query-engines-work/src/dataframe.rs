use std::sync::Arc;

use crate::data_source::registry::SourceRegistry;
use crate::logical::expr::{ColumnLogicalExpr, LogicalExpr};
use crate::logical::plan::{JoinKey, JoinType, LogicalPlan};

pub struct DataFrame {
    plan: LogicalPlan,
    sources: Arc<SourceRegistry>,
}

impl DataFrame {
    pub fn new(plan: LogicalPlan, sources: Arc<SourceRegistry>) -> Self {
        Self { plan, sources }
    }

    fn with_plan(&self, plan: LogicalPlan) -> Self {
        Self {
            plan,
            sources: self.sources.clone(),
        }
    }

    pub fn sort(&self, by: Vec<LogicalExpr>) -> Self {
        self.with_plan(self.plan.clone().sort(by))
    }

    pub fn limit(&self, limit: usize) -> Self {
        self.with_plan(self.plan.clone().limit(limit))
    }

    pub fn filter(&self, predicate: LogicalExpr) -> Self {
        self.with_plan(self.plan.clone().filter(predicate))
    }

    pub fn project(&self, exprs: Vec<LogicalExpr>) -> Self {
        self.with_plan(self.plan.clone().projection(exprs))
    }

    /// Alias for [`project`].
    pub fn select(&self, exprs: Vec<LogicalExpr>) -> Self {
        self.with_plan(self.plan.clone().projection(exprs))
    }

    pub fn agg(&self, group_by: Vec<LogicalExpr>, agg_exprs: Vec<LogicalExpr>) -> Self {
        self.with_plan(self.plan.clone().aggregate(group_by, agg_exprs))
    }

    pub fn join(&self, right: &DataFrame, how: JoinType, on: Vec<JoinKey>) -> Self {
        self.with_plan(
            self.plan
                .clone()
                .join(right.logical_plan().clone(), how, on),
        )
    }

    pub fn logical_plan(&self) -> &LogicalPlan {
        &self.plan
    }
}
