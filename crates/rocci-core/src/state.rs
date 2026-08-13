use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

/// Process-wide state accessible from lifecycle hooks and the HTTP layer.
#[derive(Clone, Default)]
pub struct ManagedState {
    inner: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ManagedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Send + Sync + 'static>(&self, value: T) {
        self.inner
            .write()
            .expect("managed state lock")
            .insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.inner
            .read()
            .expect("managed state lock")
            .get(&TypeId::of::<T>())
            .cloned()
            .and_then(|value| value.downcast().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_retrieves_typed_values() {
        let state = ManagedState::new();
        state.insert(7u32);
        assert_eq!(*state.get::<u32>().unwrap(), 7);
        assert!(state.get::<String>().is_none());
    }
}
