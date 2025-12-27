use std::time::Duration;

use byte_unit::Byte;
use cgroups_rs::fs::{cpu::CpuController, memory::MemController};

const CPU_USAGE_PREFIX: &str = "usage_usec ";

pub trait CpuControllerExt {
    fn usage(&self) -> Duration;
}

impl CpuControllerExt for CpuController {
    fn usage(&self) -> Duration {
        let stats = self.cpu().stat;

        let usage = stats
            .lines()
            .find_map(|line| line.strip_prefix(CPU_USAGE_PREFIX))
            .unwrap();
        let usage = usage.parse().unwrap();
        Duration::from_micros(usage)
    }
}

pub trait MemControllerExt {
    fn usage(&self) -> Byte;
    fn limit(&self) -> Byte;
}

impl MemControllerExt for MemController {
    fn usage(&self) -> Byte {
        let stats = self.memory_stat();

        Byte::from_u64(stats.usage_in_bytes)
    }

    fn limit(&self) -> Byte {
        let stats = self.memory_stat();

        Byte::from_u64(stats.limit_in_bytes.max(0) as u64)
    }
}
