use crate::errors::Result;
use crate::ladybug::LadybugVault;
use crate::projector::GraphProjector;
use ai_brains_store::EventStore;

pub struct GraphRebuilder<'a> {
    vault: &'a LadybugVault,
    store: &'a EventStore,
}

impl<'a> GraphRebuilder<'a> {
    pub fn new(vault: &'a LadybugVault, store: &'a EventStore) -> Self {
        Self { vault, store }
    }

    pub fn rebuild(&self) -> Result<()> {
        let projector = GraphProjector::new(self.vault);
        let events = self
            .store
            .read_all_events()
            .map_err(|e| crate::errors::GraphError::StoreError(e.to_string()))?;

        for envelope in events {
            projector.apply(&envelope)?;
        }

        Ok(())
    }
}
