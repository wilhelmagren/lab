use std::sync::Arc;

use crate::{
    data_source::{
        DataSource, RecordBatchIterator, parquet::ParquetDataSource, registry::SourceRegistry,
    },
    dataframe::DataFrame,
    logical::plan::LogicalPlan,
    planner::Planner,
};

#[derive(Clone)]
pub struct SessionContext {
    sources: Arc<SourceRegistry>,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(SourceRegistry::new()),
        }
    }

    pub fn parquet(&self, path: impl Into<String>) -> DataFrame {
        let source = ParquetDataSource::new(path);

        let name = source.name().to_owned();
        let schema = source.schema().clone();
        let source_id = self.sources.register(source);

        // TODO: support projection in scan
        let plan = LogicalPlan::scan(source_id, name, schema);

        DataFrame::new(plan, self.sources.clone())
    }

    pub fn execute(&self, df: &DataFrame) -> RecordBatchIterator {
        let logical_plan = df.logical_plan();
        println!("== Logical Plan ==\n{}", logical_plan.format(0));

        let physical_plan =
            Planner::new(self.sources.clone()).create_physical_plan(logical_plan.clone());
        println!("== Physical Plan ==\n{}", physical_plan.format(0));

        physical_plan.execute()
    }
}
