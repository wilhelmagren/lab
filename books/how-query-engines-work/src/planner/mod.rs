use std::sync::Arc;

use crate::{
    data_source::registry::SourceRegistry,
    logical::{
        expr::{LogicalExpr, LogicalExprKind},
        plan::{LogicalPlan, LogicalPlanKind},
    },
    physical::{
        expr::{
            PhysicalAggregateExpr, PhysicalBinaryExpr, PhysicalColumnExpr, PhysicalExpr,
            PhysicalExprKind, PhysicalLiteralExpr,
        },
        plan::{
            PhysicalAggregatePlan, PhysicalFilterPlan, PhysicalJoinPlan, PhysicalLimitPlan,
            PhysicalPlan, PhysicalProjectionPlan, PhysicalScanPlan,
        },
    },
};

pub struct Planner {
    sources: Arc<SourceRegistry>,
}

impl Planner {
    pub fn new(sources: Arc<SourceRegistry>) -> Self {
        Self { sources }
    }

    pub(crate) fn create_physical_plan(&self, input: impl Into<LogicalPlan>) -> PhysicalPlan {
        let input = input.into();
        match input.kind() {
            LogicalPlanKind::Limit(plan) => PhysicalLimitPlan::new(
                self.create_physical_plan(plan.input().clone()),
                plan.limit(),
            )
            .into(),
            LogicalPlanKind::Scan(plan) => PhysicalScanPlan::new(
                self.sources.get(plan.source_id()).unwrap(),
                plan.projection(),
            )
            .into(),
            LogicalPlanKind::Filter(plan) => PhysicalFilterPlan::new(
                self.create_physical_plan(plan.input().clone()),
                self.create_physical_expr(plan.input().clone(), plan.predicate().clone()),
            )
            .into(),
            LogicalPlanKind::Projection(plan) => PhysicalProjectionPlan::new(
                self.create_physical_plan(plan.input().clone()),
                plan.exprs()
                    .iter()
                    .map(|expr| self.create_physical_expr(plan.input().clone(), expr.clone()))
                    .collect::<Vec<PhysicalExpr>>(),
            )
            .into(),
            LogicalPlanKind::Aggregate(plan) => PhysicalAggregatePlan::new(
                self.create_physical_plan(plan.input().clone()),
                plan.group_exprs
                    .iter()
                    .map(|e| self.create_physical_expr(plan.input().clone(), e.clone()))
                    .collect::<Vec<PhysicalExpr>>(),
                plan.agg_exprs
                    .iter()
                    .map(|e| self.create_physical_agg_expr(plan.input().clone(), e.clone()))
                    .collect::<Vec<PhysicalExpr>>(),
            )
            .into(),
            LogicalPlanKind::Join(plan) => {
                // the join keys should be expressions instead??
                let left = self.create_physical_plan(plan.left.clone());
                let right = self.create_physical_plan(plan.right.clone());
                let left_keys = plan
                    .on
                    .iter()
                    .map(|k| plan.left.schema().index_of(&k.left).unwrap())
                    .collect::<Vec<usize>>();
                let right_keys = plan
                    .on
                    .iter()
                    .map(|k| plan.right.schema().index_of(&k.right).unwrap())
                    .collect::<Vec<usize>>();
                PhysicalJoinPlan::new(left, right, plan.how.clone(), left_keys, right_keys).into()
            }
        }
    }

    fn create_physical_expr(&self, input: LogicalPlan, expr: LogicalExpr) -> PhysicalExpr {
        match expr.kind() {
            LogicalExprKind::Column(expr) => {
                PhysicalColumnExpr::new(input.schema().index_of(expr.name()).unwrap()).into()
            }
            LogicalExprKind::Literal(expr) => {
                PhysicalLiteralExpr::new(expr.value().to_arrow_scalar()).into()
            }
            // TODO: maybe we should do specialized BinaryExpr instead, like PhysicalMultiplyExpression
            // so here we would have to match on the expr.op and dispatch during the planning phase,
            // that might be faster, cus right now we match inside the evaluate...
            LogicalExprKind::Binary(expr) => PhysicalBinaryExpr::new(
                expr.name(),
                self.create_physical_expr(input.clone(), expr.left().clone()),
                expr.op().clone(),
                self.create_physical_expr(input.clone(), expr.right().clone()),
            )
            .into(),
            // alias only affects naming during planning, as it gives a name to an expr
            // just evaluate the underlying expr, it has no specific physical repr
            LogicalExprKind::Alias(expr) => self.create_physical_expr(input, expr.expr().clone()),
            _ => unreachable!(),
        }
    }

    fn create_physical_agg_expr(&self, input: LogicalPlan, expr: LogicalExpr) -> PhysicalExpr {
        match expr.kind() {
            LogicalExprKind::Aggregate(expr) => {
                PhysicalExpr::new(PhysicalExprKind::Aggregate(PhysicalAggregateExpr {
                    op: expr.op.clone(),
                    expr: self.create_physical_expr(input, expr.expr.clone()),
                    name: expr.to_string(),
                }))
            }

            LogicalExprKind::Alias(alias) => match alias.expr().kind() {
                LogicalExprKind::Aggregate(agg) => {
                    PhysicalExpr::new(PhysicalExprKind::Aggregate(PhysicalAggregateExpr {
                        op: agg.op.clone(),
                        expr: self.create_physical_expr(input, agg.expr.clone()),
                        name: alias.name.to_owned(),
                    }))
                }

                _ => panic!("expected aggregate inside alias"),
            },

            _ => panic!("expected agg expr"),
        }
    }
}
