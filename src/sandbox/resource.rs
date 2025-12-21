use std::time::Duration;

use byte_unit::Byte;
use cgroups_rs::fs::cgroup_builder::CgroupBuilder;

pub enum Resource {
    Memory(Byte),
    Cpu { quota: Duration, period: Duration },
}

impl Resource {
    pub fn add(self, builder: CgroupBuilder) -> CgroupBuilder {
        match self {
            Resource::Memory(byte) => builder
                .memory()
                .memory_swap_limit(0)
                .memory_soft_limit(byte.as_u64() as i64)
                .memory_hard_limit(byte.as_u64() as i64)
                .done(),
            Resource::Cpu { quota, period } => builder
                .cpu()
                .quota(quota.as_micros() as i64)
                .period(period.as_micros() as u64)
                .done(),
        }
    }
}
