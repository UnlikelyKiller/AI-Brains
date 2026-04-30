use ai_brains_store::VaultConnection;

pub struct GraphVault {
    connection: VaultConnection,
}

impl GraphVault {
    pub fn new(connection: VaultConnection) -> Self {
        Self { connection }
    }

    pub fn connection(&self) -> &VaultConnection {
        &self.connection
    }
}
