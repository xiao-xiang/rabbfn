use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct StateStore {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl StateStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T>(&mut self, value: T)
    where
        T: Send + Sync + 'static,
    {
        self.values.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn get<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>().cloned())
    }

    pub fn merge_from(&mut self, other: &StateStore) {
        for (type_id, value) in &other.values {
            self.values.insert(*type_id, value.clone());
        }
    }
}
