#[derive(Debug)]
pub enum EngineError {
    MultipleInstantiation,
    CleaningFailed,
    InitializationFailed,
    ShutdownFailed,
    Unknown,
    NotInitialized,
    Duplicate,
    InvalidValue,
}
