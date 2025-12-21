mod resource;

use std::{process::Child, time::Duration};

use anyhow::Result;
use bon::Builder;
use cgroups_rs::{CgroupPid, fs::Cgroup};
pub use resource::Resource;

use crate::Metrics;

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
    pub cpu_idleness_limit: Duration,

    // TODO: need further testing
    #[builder(field = wall_time_limit / 3)]
    pub idless_limit: Duration,

    #[builder(with = |resource: Resource| -> Result<_> { Cgroup::try_from(resource) })]
    pub cgroup: Cgroup,
}

impl Sandbox {
    pub fn run(&self, child: Child) -> Result<Metrics> {
        let pid = CgroupPid::from(child.id() as u64);
        self.cgroup.add_task_by_tgid(pid)?;

        todo!()
    }
}
