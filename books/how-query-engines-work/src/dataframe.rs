use std::sync::Arc;

use crate::{data_source::registry::SourceRegistry, logical::plan::LogicalPlan};

pub struct DataFrame {
    plan: LogicalPlan,
    sources: Arc<SourceRegistry>,
}

impl DataFrame {
    pub fn new(plan: LogicalPlan, sources: Arc<SourceRegistry>) -> Self {
        Self { plan, sources }
    }
}
