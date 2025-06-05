use crate::config::AppConfig;
use azure_core::credentials::Secret;
use azure_data_cosmos::{
    CosmosClient,
    clients::{ContainerClient, DatabaseClient},
};

pub struct CosmosDBClient {
    client: CosmosClient,
    database_name: String,
    container_name: String,
}

impl CosmosDBClient {
    /// Creates a new `CosmosDBClient` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the `CosmosClient` fails to initialize with the provided configuration.
    pub fn new(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = CosmosClient::with_key(
            &config.cosmos.uri,
            Secret::from(config.cosmos.connection_string.clone()),
            None,
        )?;

        Ok(Self {
            client,
            database_name: config.cosmos.database_name.clone(),
            container_name: config.cosmos.container_name.clone(),
        })
    }

    #[must_use]
    pub fn database(&self) -> DatabaseClient {
        self.client.database_client(&self.database_name)
    }
    #[must_use]
    pub fn container(&self) -> ContainerClient {
        self.database().container_client(&self.container_name)
    }
}
