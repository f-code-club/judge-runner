use std::time::Duration;

use byte_unit::Byte;

#[derive(Debug)]
pub enum Verdict {
    Accepted,
    WrongAnswer,
    TimeLimitExceeded,
    CompilationError,
    MemoryLimitExceeded,
    RuntimeError,
    IdleTimeLimitExceeded,
}

pub struct Metrics {
    pub verdict: Verdict,
    pub run_time: Duration,
    pub stdout: String,
    pub stderr: String,
    pub memory: Byte,
}
