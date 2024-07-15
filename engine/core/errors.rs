#[derive(Debug)]
pub enum EngineError {
    MultipleInstantiation,
    CleaningFailed,
    InitializationFailed,
    Unknown,
}
