use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::data_source::{DataSource, DataSourceRef, SourceId};

#[derive(Clone, Default)]
struct Sources {
    by_id: HashMap<SourceId, DataSourceRef>,
    by_name: HashMap<String, SourceId>,
}

#[derive(Default)]
pub struct SourceRegistry {
    next_id: AtomicU64,
    sources: RwLock<Sources>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            sources: RwLock::new(Sources::default()),
        }
    }

    pub fn register<D>(&self, source: D) -> SourceId
    where
        D: DataSource + 'static,
    {
        self.register_ref(Arc::new(source))
    }

    pub fn register_ref(&self, source: DataSourceRef) -> SourceId {
        let id = SourceId::new(self.next_id.fetch_add(1, Ordering::Relaxed));
        let name = source.name().to_owned();

        let mut sources = self.sources.write().unwrap();
        sources.by_name.insert(name, id);
        sources.by_id.insert(id, source);

        id
    }

    pub fn get(&self, id: SourceId) -> Option<DataSourceRef> {
        self.sources.read().unwrap().by_id.get(&id).cloned()
    }

    pub fn get_by_name(&self, name: &str) -> Option<DataSourceRef> {
        let sources = self.sources.read().unwrap();
        let id = sources.by_name.get(name)?;
        sources.by_id.get(id).cloned()
    }

    pub fn id_by_name(&self, name: &str) -> Option<SourceId> {
        self.sources.read().unwrap().by_name.get(name).copied()
    }
}
