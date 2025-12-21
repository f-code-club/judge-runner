use std::time::Duration;

use bon::Builder;
use byte_unit::Byte;
use cgroups_rs::fs::{Cgroup, cgroup_builder::CgroupBuilder, hierarchies};
use uuid::Uuid;

const PREFIX: &str = "judge";

#[derive(Clone, Copy, Builder)]
pub struct Resource {
    #[builder(default = Byte::GIGABYTE)]
    pub memory: Byte,

    #[builder(default = Duration::from_secs(1))]
    pub cpu_quota: Duration,

    #[builder(default = Duration::from_secs(1))]
    pub cpu_period: Duration,
}

impl TryFrom<Resource> for Cgroup {
    type Error = anyhow::Error;

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

        let cgroup = builder.build(hierarchies::auto())?;
        Ok(cgroup)
    }
}
