mod resource;

use anyhow::Result;
use cgroups_rs::{CgroupPid, fs::Cgroup};
pub use resource::Resource;

use crate::Metrics;

pub struct Sandbox {
    pub cgroup: Cgroup,
    pub resource: Resource,
}

impl Sandbox {
    pub fn new(resource: Resource) -> Result<Sandbox> {
        let cgroup: Cgroup = resource.try_into()?;

        Ok(Sandbox { cgroup, resource })
    }
}
