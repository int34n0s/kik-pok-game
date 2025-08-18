use spacetimedb_sdk::credentials::CredentialFileError;

#[derive(Debug, thiserror::Error)]
pub enum RustLibError {
    #[error("error with credentials")]
    Credential {
        #[source]
        source: CredentialFileError,
    },
    #[error("error with spacetime-db sdk: {source}")]
    SpacetimeSDK {
        #[source]
        source: spacetimedb_sdk::Error,
    },

    #[error("reached wrong connection state: {0}")]
    WrongConnectionState(String),

    #[error("encountered incomplete world setup: {0}")]
    WorldSetup(String),

    #[error("resource load error: {0}")]
    ResourceLoadError(String),

    #[error("resource cast error: {0} to {1}")]
    ResourceCastError(String, String),

    #[error("resource instantiate error: {0}")]
    ResourceInstantiateError(String),
}
