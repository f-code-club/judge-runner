mod cgroup;
mod resource;

use std::{
    io,
    os::unix::process::ExitStatusExt,
    process,
    thread::sleep,
    time::{Duration, Instant},
};

use tokio::process::{Child, Command};

use byte_unit::Byte;
use cgroups_rs::{
    CgroupPid,
    fs::{Cgroup, cpu::CpuController, memory::MemController},
};
pub use resource::Resource;

use crate::{
    Verdict,
    sandbox::cgroup::{CpuControllerExt, MemControllerExt},
};

// TODO: need further tuning
const POLL: Duration = Duration::from_millis(10);
const MIN_CPU_USAGE_PER_POLL: Duration = Duration::from_millis(1);
const IDLE_TIME_LIMIT: Duration = Duration::from_millis(100);

pub struct Sandbox {
    pub cgroup: Cgroup,
    pub cpu_usage_limit: Duration,
    pub wall_time_limit: Duration,
}

impl Sandbox {
    pub fn new(resource: Resource, time_limit: Duration) -> io::Result<Sandbox> {
        Ok(Sandbox {
            cgroup: resource.try_into()?,
            cpu_usage_limit: time_limit,
            wall_time_limit: Duration::max(time_limit * 2, time_limit + Duration::from_secs(2)),
        })
    }

    pub fn spawn(&self, mut command: Command) -> io::Result<Child> {
        let cgroup = self.cgroup.clone();

        unsafe {
            command
                .pre_exec(move || {
                    let id = process::id();

                    cgroup
                        .add_task_by_tgid(CgroupPid::from(id as u64))
                        .map_err(io::Error::other)
                })
                .spawn()
        }
    }

    pub async fn monitor(&self, mut child: Child) -> io::Result<(Option<Verdict>, Duration, Byte)> {
        let Some(id) = child.id() else {
            return Err(io::Error::other("Child exited"));
        };
        self.cgroup
            .add_task_by_tgid(CgroupPid::from(id as u64))
            .map_err(io::Error::other)?;
        let cpu: &CpuController = self
            .cgroup
            .controller_of()
            .ok_or(io::Error::other("Missing cpu controller"))?;
        let memory: &MemController = self
            .cgroup
            .controller_of()
            .ok_or(io::Error::other("Missing memory controller"))?;

        let start = Instant::now();
        let mut memory_usage = Byte::default();
        let mut prev_cpu_usage = cpu.usage();
        let mut idle_start: Option<Instant> = None;

        while child.try_wait()?.is_none() {
            let cpu_usage = cpu.usage();
            memory_usage = memory_usage.max(memory.usage());

            if cpu_usage.abs_diff(prev_cpu_usage) <= MIN_CPU_USAGE_PER_POLL {
                match idle_start {
                    Some(idle_start) => {
                        if idle_start.elapsed() >= IDLE_TIME_LIMIT {
                            return Ok((
                                Some(Verdict::IdleTimeLimitExceeded),
                                cpu_usage,
                                memory_usage,
                            ));
                        }
                    }
                    None => idle_start = Some(Instant::now()),
                }
            } else {
                idle_start = None;
            }

            if cpu_usage >= self.cpu_usage_limit || start.elapsed() >= self.wall_time_limit {
                return Ok((
                    Some(Verdict::TimeLimitExceeded),
                    self.cpu_usage_limit,
                    memory_usage,
                ));
            }

            prev_cpu_usage = cpu_usage;

            sleep(POLL);
        }

        let status = child.try_wait()?.unwrap();
        if status.success() {
            return Ok((None, prev_cpu_usage, memory_usage));
        }
        match status.signal() {
            // SIGKILL
            Some(9) => Ok((
                Some(Verdict::MemoryLimitExceeded),
                prev_cpu_usage,
                memory.limit(),
            )),
            _ => Ok((Some(Verdict::RuntimeError), prev_cpu_usage, memory_usage)),
        }
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = self.cgroup.kill();
        let _ = self.cgroup.delete();
    }
}
