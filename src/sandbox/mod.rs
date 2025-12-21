mod resource;

use std::{process::Child, time::Duration};

use anyhow::Result;
use cgroups_rs::{CgroupPid, fs::Cgroup};
pub use resource::Resource;

use crate::Metrics;

pub struct Sandbox {
    pub cgroup: Cgroup,
    pub cpu_time_limit: Duration,
    pub wall_time_limit: Duration,
}

impl Sandbox {
    pub fn new(resource: Resource, time_limit: Duration) -> Result<Sandbox> {
        let cgroup: Cgroup = resource.try_into()?;
        let cpu_time_limit = time_limit;
        // TODO: need real usage to decide
        let wall_time_limit = Duration::max(time_limit * 2, time_limit + Duration::from_secs(2));

        Ok(Sandbox {
            cgroup,
            cpu_time_limit,
            wall_time_limit,
        })
    }

    pub fn run(&self, child: Child) -> Result<Metrics> {
        let pid = CgroupPid::from(child.id() as u64);
        self.cgroup.add_task_by_tgid(pid)?;

        todo!()
    }
}
