mod cgroup;
mod resource;

use std::{
    io,
    process::Child,
    thread::sleep,
    time::{Duration, Instant},
};

use cgroups_rs::{CgroupPid, fs::Cgroup};
pub use resource::Resource;

use crate::{Verdict, sandbox::cgroup::CgroupExt};

// TODO: need further tuning
const POLL: Duration = Duration::from_millis(10);
const MIN_CPU_TIME_PER_POLL: Duration = Duration::from_millis(1);
const IDLE_TIME_LIMIT: Duration = Duration::from_millis(100);

pub struct Sandbox {
    pub cgroup: Cgroup,
    pub cpu_time_limit: Duration,
    pub wall_time_limit: Duration,
}

impl Sandbox {
    pub fn new(resource: Resource, time_limit: Duration) -> io::Result<Sandbox> {
        Ok(Sandbox {
            cgroup: resource.try_into()?,
            cpu_time_limit: time_limit,
            wall_time_limit: Duration::max(time_limit * 2, time_limit + Duration::from_secs(2)),
        })
    }

    pub fn monitor(self, mut child: Child) -> io::Result<Verdict> {
        self.cgroup
            .add_task_by_tgid(CgroupPid::from(child.id() as u64))
            .map_err(io::Error::other)?;

        let start = Instant::now();
        let mut prev_cpu_time = self.cgroup.get_cpu_time();
        let mut idle_start: Option<Instant> = None;
        while child.try_wait()?.is_none() {
            let cpu_time = self.cgroup.get_cpu_time();

            if cpu_time.abs_diff(prev_cpu_time) <= MIN_CPU_TIME_PER_POLL {
                match idle_start {
                    Some(idle_start) => {
                        if idle_start.elapsed() >= IDLE_TIME_LIMIT {
                            return Ok(Verdict::IdleTimeLimitExceeded);
                        }
                    }
                    None => idle_start = Some(Instant::now()),
                }
            } else {
                idle_start = None;
            }

            if cpu_time >= self.cpu_time_limit || start.elapsed() >= self.wall_time_limit {
                return Ok(Verdict::TimeLimitExceeded);
            }

            prev_cpu_time = cpu_time;

            sleep(POLL);
        }

        // SAFETY: child must be finished at this point to exit the previous loop
        let status = child.try_wait()?.unwrap();
        if status.success() {
            // temporarily return AC
            return Ok(Verdict::Accepted);
        }
        if self.cgroup.is_out_of_memory() {
            return Ok(Verdict::MemoryLimitExceeded);
        }
        Ok(Verdict::RuntimeError)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // SAFETY: always be used with stable version of linux kernel
        self.cgroup.kill().unwrap();

        // SAFETY: no descendant is created previously by judge
        self.cgroup.delete().unwrap();
    }
}
