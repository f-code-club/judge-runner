use std::{io, time::Duration};

use byte_unit::Byte;
use cgroups_rs::fs::{Cgroup, cgroup_builder::CgroupBuilder, hierarchies};
use uuid::Uuid;

const PREFIX: &str = "judge";

#[derive(Clone, Copy, Hash)]
pub struct Resource {
    pub memory: Byte,
    pub cpu_quota: Duration,
    pub cpu_period: Duration,
}

impl Default for Resource {
    fn default() -> Self {
        Resource {
            memory: Byte::GIGABYTE,
            cpu_quota: Duration::from_millis(100),
            cpu_period: Duration::from_millis(100),
        }
    }
}

impl TryFrom<Resource> for Cgroup {
    type Error = io::Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        let builder = CgroupBuilder::new(&format!("{}/{}", PREFIX, Uuid::new_v4()));

        let memory = resource.memory.as_u64() as i64;
        let builder = builder
            .memory()
            .memory_swap_limit(0)
            .memory_soft_limit(memory)
            .memory_hard_limit(memory)
            .done();

        let quota = resource.cpu_quota.as_micros() as i64;
        let period = resource.cpu_period.as_micros() as u64;
        let builder = builder.cpu().quota(quota).period(period).done();

        let cgroup = builder
            .build(hierarchies::auto())
            .map_err(io::Error::other)?;
        Ok(cgroup)
    }
}
