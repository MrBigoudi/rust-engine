#[derive(Debug)]
pub enum EngineError {
    MultipleInstantiation,
    InitializationFailed,
    ShutdownFailed,
    Unknown,
    NotInitialized,
    Duplicate,
    InvalidValue,
    NotImplemented,
    VulkanFailed,
    AccessFailed,
    Synchronisation,
    UpdateFailed,
}
