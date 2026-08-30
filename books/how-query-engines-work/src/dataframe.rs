use std::sync::Arc;

use crate::data_source::registry::SourceRegistry;
use crate::logical::{
    expr::Expr,
    plan::{JoinKey, JoinType, LogicalPlan},
};

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

    pub fn limit(&self, limit: usize) -> Self {
        self.with_plan(self.plan.clone().limit(limit))
    }

    pub fn filter(&self, predicate: Expr) -> Self {
        self.with_plan(self.plan.clone().filter(predicate))
    }

    pub fn project(&self, exprs: Vec<Expr>) -> Self {
        self.with_plan(self.plan.clone().projection(exprs))
    }

    /// Alias for [`project`].
    pub fn select(&self, exprs: Vec<Expr>) -> Self {
        self.with_plan(self.plan.clone().projection(exprs))
    }

    pub fn agg(&self, group_by: Vec<Expr>, agg_exprs: Vec<Expr>) -> Self {
        self.with_plan(self.plan.clone().aggregate(group_by, agg_exprs))
    }

    pub fn join(&self, right: DataFrame, how: JoinType, on: Vec<JoinKey>) -> Self {
        self.with_plan(self.plan.clone().join(right.get_plan().clone(), how, on))
    }

    pub fn get_plan(&self) -> &LogicalPlan {
        &self.plan
    }

    pub fn print_plan(&self) {
        println!("{}", self.plan.format(0));
    }
}
