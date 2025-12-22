mod resource;

use std::{
    os::unix::process::ExitStatusExt,
    process::{Child, ExitStatus},
    time::{Duration, Instant},
};

use anyhow::Result;
use bon::Builder;
use cgroups_rs::{
    CgroupPid,
    fs::{
        Cgroup,
        cpu::CpuController,
        memory::{MemController, Memory},
    },
};
use nix::sys::signal::Signal;
pub use resource::Resource;

use crate::Metrics;

const CPU_USAGE_PREFIX: &str = "usage_usec ";

#[derive(Builder)]
pub struct Sandbox {
    #[builder(start_fn)]
    pub cpu_time_limit: Duration,

    #[builder(start_fn)]
    pub poll: Duration,

    // TODO: need further testing
    #[builder(field = Duration::max(cpu_time_limit * 2, cpu_time_limit + Duration::from_secs(2)))]
    pub wall_time_limit: Duration,

    // TODO: need further testing
    #[builder(field = poll / 10)]
    pub min_cpu_time_per_poll: Duration,

    // TODO: need further testing
    #[builder(field = wall_time_limit / 3)]
    pub idle_time_limit: Duration,

    #[builder(with = |resource: Resource| -> Result<_> { Cgroup::try_from(resource) })]
    pub cgroup: Cgroup,
}

impl Sandbox {
    fn get_cpu_time(&self) -> Duration {
        // SAFETY: there must be cpu controller for cgroup v2
        let cpu_controller: &CpuController = self.cgroup.controller_of().unwrap();
        let stats = cpu_controller.cpu().stat;

        // SAFETY: there must be cpu usage for valid cgroup
        let usage = stats
            .lines()
            .find_map(|line| line.strip_suffix(CPU_USAGE_PREFIX))
            .unwrap();
        // SAFETY: cpu usage must be duration in microsecond
        let usage = usage.parse().unwrap();
        Duration::from_micros(usage)
    }

    fn try_wait(&self, child: &mut Child) -> Result<Option<Metrics>> {
        let Some(status) = child.try_wait()? else {
            return Ok(None);
        };
        if status.success() {
            // temporarily return AC
            return Ok(Some(Metrics::Accepted));
        }

        // SAFETY: there must be memory controller for cgroup v2
        let cpu_controller: &MemController = self.cgroup.controller_of().unwrap();
        let stats = cpu_controller.memory_stat();
        if stats.oom_control.oom_kill > 0 {
            return Ok(Some(Metrics::MemoryLimitExceeded));
        }

        // SAFETY: process must be terminated with valid signal
        let metrics = match status.signal().map(|raw| Signal::try_from(raw).unwrap()) {
            Some(Signal::SIGKILL) => {
                if stats.usage_in_bytes as i64 > stats.limit_in_bytes {
                    Metrics::MemoryLimitExceeded
                } else {
                    Metrics::RuntimeError
                }
            }
            _ => Metrics::RuntimeError,
        };
        Ok(Some(metrics))
    }

    fn is_idle(
        &self,
        cpu_time: Duration,
        prev_cpu_time: Duration,
        idle_start: &mut Option<Instant>,
    ) -> bool {
        if cpu_time.abs_diff(prev_cpu_time) >= self.min_cpu_time_per_poll {
            *idle_start = None;
            return false;
        }
        match idle_start {
            Some(idle_start) => idle_start.elapsed() >= self.idle_time_limit,
            None => {
                *idle_start = Some(Instant::now());
                false
            }
        }
    }

    pub fn run(&self, mut child: Child) -> Result<Metrics> {
        let pid = CgroupPid::from(child.id() as u64);
        self.cgroup.add_task_by_tgid(pid)?;

        let start = Instant::now();
        let mut prev_cpu_time = self.get_cpu_time();
        let mut idle_start: Option<Instant> = None;

        loop {
            if let Some(metrics) = self.try_wait(&mut child)? {
                return Ok(metrics);
            }
            let cpu_time = self.get_cpu_time();

            if self.is_idle(cpu_time, prev_cpu_time, &mut idle_start) {
                return Ok(Metrics::IdleTimeLimitExceeded);
            }

            if cpu_time >= self.cpu_time_limit || start.elapsed() >= self.wall_time_limit {
                return Ok(Metrics::TimeLimitExceeded);
            }

            prev_cpu_time = cpu_time;
        }
    }
}
