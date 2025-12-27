use std::time::Duration;

use byte_unit::Byte;
use cgroups_rs::fs::{Cgroup, cpu::CpuController, memory::MemController};

const CPU_USAGE_PREFIX: &str = "usage_usec ";

pub trait CgroupExt {
    fn get_cpu_time(&self) -> Duration;
    fn get_memory_usage(&self) -> Byte;
    fn get_memory_limit(&self) -> Byte;
}

impl CgroupExt for Cgroup {
    fn get_cpu_time(&self) -> Duration {
        let cpu_controller: &CpuController = self.controller_of().unwrap();
        let stats = cpu_controller.cpu().stat;

        // SAFETY: there must be cpu usage for valid cgroup
        let usage = stats
            .lines()
            .find_map(|line| line.strip_prefix(CPU_USAGE_PREFIX))
            .unwrap();
        // SAFETY: cpu usage must be duration in microsecond
        let usage = usage.parse().unwrap();
        Duration::from_micros(usage)
    }

    fn get_memory_usage(&self) -> Byte {
        // SAFETY: there must be memory controller for cgroup v2
        let memory_controller: &MemController = self.controller_of().unwrap();
        let stats = memory_controller.memory_stat();

        Byte::from_u64(stats.usage_in_bytes)
    }

    fn get_memory_limit(&self) -> Byte {
        // SAFETY: there must be memory controller for cgroup v2
        let memory_controller: &MemController = self.controller_of().unwrap();
        let stats = memory_controller.memory_stat();

        Byte::from_u64(stats.limit_in_bytes.max(0) as u64)
    }
}
