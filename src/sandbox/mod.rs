mod resource;

use std::{process::Child, time::Duration};

use anyhow::Result;
use cgroups_rs::{CgroupPid, fs::Cgroup};
pub use resource::Resource;

use crate::Metrics;

pub struct Sandbox {
    pub cgroup: Cgroup,
    pub time_limit: Duration,
}

impl Sandbox {
    pub fn new(resource: Resource, time_limit: Duration) -> Result<Sandbox> {
        let cgroup: Cgroup = resource.try_into()?;

        Ok(Sandbox { cgroup, time_limit })
    }

    pub fn run(&self, child: Child) -> Result<Metrics> {
        let pid = CgroupPid::from(child.id() as u64);
        self.cgroup.add_task_by_tgid(pid)?;

        todo!()
    }
}
