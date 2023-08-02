use std::time::Duration;

/// Describes a possible timeout for a mapping
#[derive(Debug, Clone, Copy)]
pub enum MaybeTimeout {
    /// Indicates that a mapping should never time out
    Never,
    /// Indicates that a mapping should time out after a given duration
    After {
        duration: Duration,
        start: std::time::Instant,
    },
}