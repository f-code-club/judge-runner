use std::time::Duration;

use byte_unit::Byte;

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    CompilationError,
    MemoryLimitExceeded,
    RuntimeError,
    IdleTimeLimitExceeded,
}

#[derive(Debug)]
pub struct Metrics {
    pub verdict: Verdict,
    pub run_time: Duration,
    pub memory_usage: Byte,
    pub stdout: String,
    pub stderr: String,
}
