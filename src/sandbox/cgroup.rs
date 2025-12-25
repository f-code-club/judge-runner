use std::time::Duration;

use cgroups_rs::fs::{Cgroup, cpu::CpuController};

const CPU_USAGE_PREFIX: &str = "usage_usec ";

pub trait CgroupExt {
    fn get_cpu_time(&self) -> Duration;
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
}
