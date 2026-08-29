use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::data_source::{DataSource, DataSourceRef, SourceId};

pub struct SourceRegistry {
    next_id: AtomicU64,
    sources: RwLock<HashMap<SourceId, DataSourceRef>>,
}

impl SourceRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            sources: RwLock::new(HashMap::new()),
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
        self.sources.write().unwrap().insert(id, source);
        id
    }

    pub fn get(&self, id: SourceId) -> Option<DataSourceRef> {
        self.sources.read().unwrap().get(&id).cloned()
    }
}

impl std::default::Default for SourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
