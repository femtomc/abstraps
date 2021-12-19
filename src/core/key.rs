use std::any::{Any, TypeId};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

pub trait Key {
    fn eq(&self, other: &dyn Key) -> bool;
    fn hash(&self) -> u64;
    fn as_any(&self) -> &dyn Any;
}

impl<T: Eq + Hash + 'static> Key for T {
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

    fn as_any(&self) -> &dyn Any {
        self
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
