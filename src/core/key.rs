use downcast_rs::{impl_downcast, Downcast};
use std::any::{Any, TypeId};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

pub trait Key: Downcast {
    fn eq(&self, other: &dyn Key) -> bool;
    fn hash(&self) -> u64;
}
impl_downcast!(Key);

impl<T> Key for T
where
    T: Eq + Hash + 'static,
{
    fn eq(&self, other: &dyn Key) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            return self == other;
        }
        false
    }

    fn hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        Hash::hash(&(TypeId::of::<T>(), self), &mut h);
        h.finish()
    }
}

impl PartialEq for Box<dyn Key> {
    fn eq(&self, other: &Self) -> bool {
        Key::eq(self.as_ref(), other.as_ref())
    }
}

impl Eq for Box<dyn Key> {}

impl Hash for Box<dyn Key> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let key_hash = Key::hash(self.as_ref());
        state.write_u64(key_hash);
    }
}

fn into_key(key: impl Eq + Hash + 'static) -> Box<dyn Key> {
    Box::new(key)
}
