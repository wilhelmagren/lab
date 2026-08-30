use std::sync::Arc;

use arrow::datatypes::SchemaRef;

use crate::data_source::RecordBatchIterator;

#[derive(Clone)]
pub enum PhysicalPlanKind {}

#[derive(Clone)]
pub struct PhysicalPlan(Arc<PhysicalPlanKind>);

impl PhysicalPlan {
    pub fn new(plan: PhysicalPlanKind) -> Self {
        Self(Arc::new(plan))
    }

    fn kind(&self) -> &PhysicalPlanKind {
        self.0.as_ref()
    }

    pub fn schema(&self) -> &SchemaRef {
        match self.kind() {
            _ => todo!(),
        }
    }

    pub fn inputs(&self) -> Vec<&PhysicalPlan> {
        match self.kind() {
            _ => todo!(),
        }
    }

    pub fn execute(&self) -> RecordBatchIterator<'_> {
        todo!()
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
