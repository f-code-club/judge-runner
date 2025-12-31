use std::hash::{DefaultHasher, Hash, Hasher};

use uuid::Uuid;

pub fn random<T: Hash>(data: T) -> u64 {
    let mut hasher = DefaultHasher::default();
    data.hash(&mut hasher);
    Uuid::new_v4().hash(&mut hasher);

    hasher.finish()
}
