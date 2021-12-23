use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

/// Represents a trait object's vtable pointer. You shouldn't need to use this as a
/// consumer of the crate but it is required for macro expansion.
#[doc(hidden)]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VTable(*const ());

impl VTable {
    pub fn none() -> VTable {
        VTable(ptr::null())
    }
}

unsafe impl Send for VTable {}
unsafe impl Sync for VTable {}

/// Represents a trait object's layout. You shouldn't need to use this as a
/// consumer of the crate but it is required for macro expansion.
#[doc(hidden)]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TraitObject {
    pub data: *const (),
    pub vtable: VTable,
}

/// Obtain the vtable for a type/trait pair. You shouldn't need to use this as a
/// consumer of the crate but it is required for macro expansion.
#[doc(hidden)]
#[macro_export]
macro_rules! vtable_for {
    ($x:ty as $y:ty) => {{
        let x = ::std::ptr::null::<$x>() as *const $y;
        #[allow(unused_unsafe)]
        unsafe {
            ::std::mem::transmute::<_, $crate::TraitObject>(x).vtable
        }
    }};
}

#[macro_export]
macro_rules! mopo {
    ($name:ty) => {
        impl $name {
            pub fn query_ref<U: ::std::any::Any + ?Sized>(&self) -> Option<&U> {
                if let Some(vtable) = self.query_vtable(::std::any::TypeId::of::<U>()) {
                    unsafe {
                        let data = self as *const Self;
                        let u = $crate::TraitObject {
                            data: data as *const (),
                            vtable,
                        };
                        Some(*::std::mem::transmute::<_, &&U>(&u))
                    }
                } else {
                    None
                }
            }

            pub fn query_mut<U: ::std::any::Any + ?Sized>(&mut self) -> Option<&mut U> {
                if let Some(vtable) = self.query_vtable(::std::any::TypeId::of::<U>()) {
                    unsafe {
                        let data = self as *mut Self;
                        let mut u = $crate::TraitObject {
                            data: data as *const (),
                            vtable,
                        };
                        Some(*::std::mem::transmute::<_, &mut &mut U>(&mut u))
                    }
                } else {
                    None
                }
            }

            pub fn query<U: ::std::any::Any + ?Sized>(
                self: Box<Self>,
            ) -> ::std::result::Result<Box<U>, Box<Self>> {
                if let Some(vtable) = self.query_vtable(::std::any::TypeId::of::<U>()) {
                    unsafe {
                        let data = Box::into_raw(self);
                        let mut u = $crate::TraitObject {
                            data: data as *const (),
                            vtable,
                        };
                        Ok(Box::from_raw(*::std::mem::transmute::<_, &mut *mut U>(
                            &mut u,
                        )))
                    }
                } else {
                    Err(self)
                }
            }

            pub fn query_arc<U: ::std::any::Any + ?Sized>(
                self_: ::std::sync::Arc<Self>,
            ) -> ::std::result::Result<::std::sync::Arc<U>, ::std::sync::Arc<Self>> {
                if let Some(vtable) = self_.query_vtable(::std::any::TypeId::of::<U>()) {
                    unsafe {
                        let data = ::std::sync::Arc::into_raw(self_);
                        let mut u = $crate::TraitObject {
                            data: data as *const (),
                            vtable,
                        };
                        Ok(::std::sync::Arc::from_raw(*::std::mem::transmute::<
                            _,
                            &mut *mut U,
                        >(&mut u)))
                    }
                } else {
                    Err(self_)
                }
            }

            pub fn query_rc<U: ::std::any::Any + ?Sized>(
                self_: ::std::rc::Rc<Self>,
            ) -> ::std::result::Result<::std::rc::Rc<U>, ::std::rc::Rc<Self>> {
                if let Some(vtable) = self_.query_vtable(::std::any::TypeId::of::<U>()) {
                    unsafe {
                        let data = ::std::rc::Rc::into_raw(self_);
                        let mut u = $crate::TraitObject {
                            data: data as *const (),
                            vtable,
                        };
                        Ok(::std::rc::Rc::from_raw(*::std::mem::transmute::<
                            _,
                            &mut *mut U,
                        >(&mut u)))
                    }
                } else {
                    Err(self_)
                }
            }
            pub fn obj_partial_eq(&self, other: &Self) -> bool {
                if let Some(x) = self.query_ref::<dyn $crate::ObjectPartialEq>() {
                    x.obj_eq(other.query_ref().unwrap())
                } else {
                    std::ptr::eq(
                        self as *const _ as *const (),
                        other as *const _ as *const (),
                    )
                }
            }
            pub fn obj_partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                if let Some(x) = self.query_ref::<dyn $crate::ObjectPartialOrd>() {
                    x.obj_partial_cmp(other.query_ref().unwrap())
                } else {
                    None
                }
            }
        }
        impl ::std::clone::Clone for Box<$name> {
            fn clone(&self) -> Self {
                (**self).to_owned()
            }
        }
        impl ::std::borrow::ToOwned for $name {
            type Owned = Box<$name>;
            fn to_owned(&self) -> Box<$name> {
                self.query_ref::<dyn $crate::ObjectClone>()
                    .expect("Object not clonable!")
                    .obj_clone()
                    .query::<$name>()
                    .unwrap()
            }
        }
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                if let Some(o) = self.query_ref::<dyn std::fmt::Debug>() {
                    o.fmt(f)
                } else {
                    writeln!(f, "Object {{ <no `Debug` implementation> }}")
                }
            }
        }
        impl ::std::cmp::PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                // Require `Eq` rather than `PartialEq` as this allows `Object`s to be used as
                // key in hash maps
                if let Some(x) = self.query_ref::<dyn $crate::ObjectEq>() {
                    x.obj_eq(other.query_ref().unwrap())
                } else {
                    // This trivially meets the requirements of `Eq`
                    std::ptr::eq(
                        self as *const _ as *const (),
                        other as *const _ as *const (),
                    )
                }
            }
        }
        impl ::std::cmp::Eq for $name {}
        impl ::std::cmp::PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl ::std::cmp::Ord for $name {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                if let Some(x) = self.query_ref::<dyn $crate::ObjectOrd>() {
                    if let Some(o) = x.obj_cmp(other.query_ref().unwrap()) {
                        return o;
                    }
                }
                Ord::cmp(&(self as *const Self), &(other as *const Self))
            }
        }
        impl ::std::hash::Hash for $name {
            fn hash<H: ::std::hash::Hasher>(&self, state: &mut H) {
                if let Some(x) = self.query_ref::<dyn $crate::ObjectHash>() {
                    x.obj_hash(state)
                } else {
                    state.write_usize(self as *const Self as *const () as usize)
                }
            }
        }
    };
}

/// This trait is the primary function of the library. `Object` trait objects
/// can be freely queried for any other trait, allowing conversion between
/// trait objects.
pub unsafe trait Object: Any {
    /// This is implemented by the `interfaces!` macro, and should never be
    /// manually implemented.
    #[doc(hidden)]
    fn query_vtable(&self, id: TypeId) -> Option<VTable>;
}

/// You can use this trait to ensure that a type implements a trait as an
/// interface. This means the type declared the trait in its `interfaces!(...)`
/// list, and guarantees that querying an `Object` of that type for the trait
/// will always succeed.
///
/// When using `HasInterface<SomeTrait>` in a generic bound, you should also
/// specify `SomeTrait` as a bound. While `HasInterface<SomeTrait>` is a more
/// stringent requirement than, and in practice implies `SomeTrait`, the
/// compiler cannot deduce that because it is enforced through macros rather
/// than the type system.
pub unsafe trait HasInterface<I: ?Sized> {}

mopo!(dyn Object);

/// This is an object-safe version of `Clone`, which is automatically
/// implemented for all `Clone + Object` types. This is a support trait used to
/// allow `Object` trait objects to be clonable.
pub trait ObjectClone {
    fn obj_clone(&self) -> Box<dyn Object>;
}
impl<T: Clone + Object> ObjectClone for T {
    fn obj_clone(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }
}

/// This is an object-safe version of `PartialEq`, which is automatically
/// implemented for all `PartialEq + Object` types. This is a support trait used to
/// allow `Object` trait objects to be comparable in this way.
pub trait ObjectPartialEq {
    fn obj_eq(&self, other: &dyn Object) -> bool;
}
impl<T: PartialEq + Object> ObjectPartialEq for T {
    fn obj_eq(&self, other: &dyn Object) -> bool {
        if let Some(o) = other.query_ref::<Self>() {
            self == o
        } else {
            false
        }
    }
}

/// This is an object-safe version of `Eq`, which is automatically
/// implemented for all `Eq + Object` types. This is a support trait used to
/// allow `Object` trait objects to be comparable in this way.
pub trait ObjectEq: ObjectPartialEq {}
impl<T: Eq + Object> ObjectEq for T {}

/// This is an object-safe version of `PartialOrd`, which is automatically
/// implemented for all `PartialOrd + Object` types. This is a support trait used to
/// allow `Object` trait objects to be comparable in this way.
pub trait ObjectPartialOrd {
    fn obj_partial_cmp(&self, other: &dyn Object) -> Option<std::cmp::Ordering>;
}
impl<T: PartialOrd + Object> ObjectPartialOrd for T {
    fn obj_partial_cmp(&self, other: &dyn Object) -> Option<std::cmp::Ordering> {
        if let Some(o) = other.query_ref::<Self>() {
            self.partial_cmp(o)
        } else {
            None
        }
    }
}

/// This is an object-safe version of `Ord`, which is automatically
/// implemented for all `Ord + Object` types. This is a support trait used to
/// allow `Object` trait objects to be comparable in this way.
pub trait ObjectOrd {
    fn obj_cmp(&self, other: &dyn Object) -> Option<std::cmp::Ordering>;
}
impl<T: Ord + Object> ObjectOrd for T {
    fn obj_cmp(&self, other: &dyn Object) -> Option<std::cmp::Ordering> {
        other.query_ref::<Self>().map(|o| self.cmp(o))
    }
}

/// This is an object-safe version of `Hash`, which is automatically
/// implemented for all `Hash + Object` types. This is a support trait used to
/// allow `Object` trait objects to be comparable in this way.
///
/// Note: `Object`s are not guaranteed to hash to the same value as their
/// underlying type.
pub trait ObjectHash {
    fn obj_hash(&self, state: &mut dyn Hasher);
}
impl<T: Hash + Object> ObjectHash for T {
    fn obj_hash(&self, state: &mut dyn Hasher) {
        let mut h = DefaultHasher::new();
        self.hash(&mut h);
        state.write_u64(h.finish());
    }
}

#[macro_export(local_inner_macros)]
macro_rules! interfaces {
    (@unbracket $(($($v:tt)*))*) => ($($($v)*)*);
    (@inner $imp:tt $cond:tt $name:ty: $($iface:ty),+ {}) => (
        interfaces!(@unbracket $imp ($crate::HasInterface<$name> for $name) $cond ({}));
        interfaces!(@unbracket $imp ($crate::HasInterface<dyn $crate::Object> for $name) $cond ({}));
        $(interfaces!(@unbracket $imp ($crate::HasInterface<$iface> for $name) $cond ({}));)*
        interfaces!(@unbracket $imp ($crate::Object for $name) $cond ({
            fn query_vtable(&self, id: ::std::any::TypeId) -> Option<$crate::VTable> {
                if id == ::std::any::TypeId::of::<$name>() {
                    Some($crate::VTable::none())
                } else if id == ::std::any::TypeId::of::<dyn $crate::Object>() {
                    Some(vtable_for!($name as dyn $crate::Object))
                } else $(if id == ::std::any::TypeId::of::<$iface>() {
                    Some(vtable_for!($name as $iface))
                } else)* {
                    // If "dynamic" feature is enabled, fall back to
                    // looking in the registry
                    { $crate::find_in_registry::<$name>(id) }
                }
            }
        }));
    );
    (@imp ($($result:tt)*) $name:ty: $($iface:ty),+ $(where $($cond:tt)*)*) => (
        interfaces!(@inner (unsafe impl<$($result)*>) ($(where $($cond)*)*) $name: $($iface),+ {});
    );
    (@parse < $($rest:tt)*) => (
        interfaces!(@parseArg () $($rest)*);
    );
    (@parse $($rest:tt)*) => (
        interfaces!(@imp () $($rest)*);
    );
    (@parseArg ($($result:tt)*) $name:ident , $($rest:tt)*) => (
        interfaces!(@parseArg ($($result)* $name ,) $($rest)*);
    );
    (@parseArg ($($result:tt)*) $name:ident : $($rest:tt)*) => (
        interfaces!(@parseBound ($($result)* $name : ) $($rest)*);
    );
    (@parseArg ($($result:tt)*) $name:ident > $($rest:tt)*) => (
        interfaces!(@imp ($($result)* $name) $($rest)*);
    );
    (@parseBound ($($result:tt)*) $bound:tt + $($rest:tt)*) => (
        interfaces!(@parseBound ($($result)* $bound +) $($rest)*);
    );
    (@parseBound ($($result:tt)*) $bound:tt , $($rest:tt)*) => (
        interfaces!(@parseArg ($($result)* $bound ,) $($rest)*);
    );
    (@parseBound ($($result:tt)*) $bound:tt > $($rest:tt)*) => (
        interfaces!(@imp ($($result)* $bound) $($rest)*);
    );
    (< $($rest:tt)*) => (
        interfaces!(@parse < $($rest)*);
    );
    ($x:ty: $($rest:tt)*) => (
        interfaces!(@parse $x: $($rest)*);
    );
    (@expand2 ($name:ty) ($($rest:tt)*)) => (
        interfaces!($name $($rest)*);
    );
    (@expand {$($name:ty),*} $rest:tt) => (
        $( interfaces!(@expand2 ($name) $rest); )*
    );
    ({$($name:ty),*} $($rest:tt)*) => (
        interfaces!(@expand {$($name),*} ($($rest)*));
    );
}

// Integral types
interfaces!({
    bool, i8, u8, i16, u16, i32, u32, i64, u64, char
}: dyn ObjectClone, dyn Debug, dyn Display, dyn ObjectPartialEq, dyn ObjectPartialOrd, dyn ObjectEq, dyn ObjectOrd, dyn ObjectHash, dyn ToString);

// Floating point types
interfaces!({
    f32, f64
}: dyn ObjectClone, dyn Debug, dyn Display, dyn ObjectPartialEq, dyn ObjectPartialOrd, dyn ToString);

// Strings
interfaces!(
    String: dyn ObjectClone,
    dyn Debug,
    dyn Display,
    dyn ObjectPartialEq,
    dyn ObjectPartialOrd,
    dyn ObjectEq,
    dyn ObjectOrd,
    dyn ObjectHash,
    dyn ToString
);

// Paths
interfaces!(
    PathBuf: dyn ObjectClone,
    dyn Debug,
    dyn ObjectPartialEq,
    dyn ObjectPartialOrd,
    dyn ObjectEq,
    dyn ObjectOrd,
    dyn ObjectHash
);

// Vecs
interfaces!({
    Vec<bool>, Vec<i8>, Vec<u8>, Vec<i16>, Vec<u16>, Vec<i32>, Vec<u32>, Vec<i64>, Vec<u64>, Vec<char>
}: dyn ObjectClone, dyn Debug, dyn ObjectPartialEq, dyn ObjectPartialOrd, dyn ObjectEq, dyn ObjectOrd, dyn ObjectHash);
interfaces!({
    Vec<f32>, Vec<f64>
}: dyn ObjectClone, dyn Debug, dyn ObjectPartialEq, dyn ObjectPartialOrd);
interfaces!({
    Vec<String>, Vec<PathBuf>
}: dyn ObjectClone, dyn Debug, dyn ObjectPartialEq, dyn ObjectPartialOrd, dyn ObjectEq, dyn ObjectOrd, dyn ObjectHash);

type RegistryKey = (TypeId, TypeId);

#[doc(hidden)]
#[derive(Default, Clone)]
pub struct Registry {
    entries: HashMap<RegistryKey, VTable>,
}

impl Registry {
    #[doc(hidden)]
    pub unsafe fn register<Type: 'static + ?Sized, Trait: 'static + ?Sized>(
        &mut self,
        vtable: VTable,
    ) {
        self.entries
            .insert((TypeId::of::<Type>(), TypeId::of::<Trait>()), vtable);
    }
    fn find<Type: 'static + ?Sized>(&self, trait_id: TypeId) -> Option<VTable> {
        self.entries.get(&(TypeId::of::<Type>(), trait_id)).cloned()
    }
}

// The global registry can be dynamically updated, so must be protected
// by an RwLock.
#[derive(Default)]
struct GlobalRegistry {
    registry: Arc<Registry>,
}

// But we need read access to be *super* fast, since it's accessed
// on every query, so we store a local reference that doesn't require
// any synchronisation to access.
struct LocalRegistry {
    registry: Arc<Registry>,
    version: usize,
}

lazy_static! {
    static ref GLOBAL_REGISTRY: RwLock<GlobalRegistry> = RwLock::default();
}
static GLOBAL_REGISTRY_VERSION: AtomicUsize = AtomicUsize::new(0);

impl LocalRegistry {
    fn obtain() -> Self {
        // Obtain read access to global registry
        let lock_guard = GLOBAL_REGISTRY.read().unwrap();

        // Keep a local reference to the registry
        LocalRegistry {
            registry: lock_guard.registry.clone(),
            version: GLOBAL_REGISTRY_VERSION.load(Ordering::Acquire),
        }
    }
}

thread_local! {
    static LOCAL_REGISTRY: RefCell<LocalRegistry> = RefCell::new(LocalRegistry::obtain());
}

// *Fast* read-only access to the registry
fn with_registry<R, F: FnOnce(&Registry) -> R>(f: F) -> R {
    LOCAL_REGISTRY.with(|registry| {
        let mut registry = registry.borrow_mut();

        // Check for updates
        if GLOBAL_REGISTRY_VERSION.load(Ordering::Acquire) != registry.version {
            *registry = LocalRegistry::obtain();
        }

        // Access to registry
        f(&registry.registry)
    })
}

// Slower write access to the registry
#[doc(hidden)]
pub fn with_registry_mut<R, F: FnOnce(&mut Registry) -> R>(f: F) -> R {
    // Obtain write access to global registry
    let mut lock_guard = GLOBAL_REGISTRY.write().unwrap();

    // Allow caller to mutate registry
    let result = f(Arc::make_mut(&mut lock_guard.registry));

    // Notify readers that the registry has changed
    GLOBAL_REGISTRY_VERSION.fetch_add(1, Ordering::AcqRel);

    result
}

#[doc(hidden)]
pub fn find_in_registry<Type: 'static + ?Sized>(trait_id: TypeId) -> Option<VTable> {
    with_registry(|registry| registry.find::<Type>(trait_id))
}

#[macro_export]
macro_rules! dynamic_interfaces {
    ($($name:ty: $($iface:ty),+;)*) => (
        $crate::with_registry_mut(|registry| { unsafe { $(
            registry.register::<$name, $name>($crate::VTable::none());
            registry.register::<$name, dyn $crate::Object>(vtable_for!($name as dyn $crate::Object));
            $(
            registry.register::<$name, $iface>(vtable_for!($name as $iface));
            )+
        )* } });
    )
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::rc::Rc;
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct Bar;
    interfaces!(Bar: dyn Foo, dyn super::ObjectClone, dyn Debug, dyn Custom);

    trait Foo: Debug {
        fn test(&self) -> bool {
            false
        }
    }
    trait Foo2: Debug {}
    impl Foo for Bar {
        fn test(&self) -> bool {
            true
        }
    }
    impl Foo2 for Bar {}

    #[derive(Debug, Clone)]
    struct GenericBar<T>(T);
    interfaces!(<T: Debug + 'static> GenericBar<T>: dyn super::ObjectClone, dyn Debug where T: Clone);

    #[test]
    fn test_ref() {
        let x = Box::new(Bar) as Box<dyn super::Object>;
        let foo: Option<&dyn Foo> = x.query_ref();
        assert!(foo.is_some());
        assert!(foo.unwrap().test());
        let foo2: Option<&dyn Foo2> = x.query_ref();
        assert!(foo2.is_none());
        let bar: Option<&Bar> = x.query_ref();
        assert!(bar.is_some());
    }

    #[test]
    fn test_mut() {
        let mut x = Box::new(Bar) as Box<dyn super::Object>;
        {
            let foo = x.query_mut::<dyn Foo>();
            assert!(foo.is_some());
            assert!(foo.unwrap().test());
        }
        {
            let foo2 = x.query_mut::<dyn Foo2>();
            assert!(foo2.is_none());
        }
        {
            let bar = x.query_mut::<Bar>();
            assert!(bar.is_some());
        }
    }

    #[test]
    fn test_owned() {
        let x = Box::new(Bar) as Box<dyn super::Object>;
        let foo: Result<Box<dyn Foo>, _> = x.clone().query();
        assert!(foo.is_ok());
        assert!(foo.unwrap().test());
        let foo2: Result<Box<dyn Foo2>, _> = x.clone().query();
        assert!(foo2.is_err());
        let bar: Result<Box<Bar>, _> = x.clone().query();
        assert!(bar.is_ok());
    }

    #[test]
    fn test_rc() {
        let x = Rc::new(Bar) as Rc<dyn super::Object>;
        let foo: Result<Rc<dyn Foo>, _> = <dyn super::Object>::query_rc(x.clone());
        assert!(foo.is_ok());
        assert!(foo.unwrap().test());
        let foo2: Result<Rc<dyn Foo2>, _> = <dyn super::Object>::query_rc(x.clone());
        assert!(foo2.is_err());
        let bar: Result<Rc<Bar>, _> = <dyn super::Object>::query_rc(x.clone());
        assert!(bar.is_ok());
    }

    #[test]
    fn test_arc() {
        let x = Arc::new(Bar) as Arc<dyn super::Object>;
        let foo: Result<Arc<dyn Foo>, _> = <dyn super::Object>::query_arc(x.clone());
        assert!(foo.is_ok());
        assert!(foo.unwrap().test());
        let foo2: Result<Arc<dyn Foo2>, _> = <dyn super::Object>::query_arc(x.clone());
        assert!(foo2.is_err());
        let bar: Result<Arc<Bar>, _> = <dyn super::Object>::query_arc(x.clone());
        assert!(bar.is_ok());
    }

    trait Custom: super::Object {}
    impl Custom for Bar {}
    mopo!(dyn Custom);

    #[test]
    fn test_derived() {
        let x = Box::new(Bar) as Box<dyn Custom>;
        let foo: Result<Box<dyn Foo>, _> = x.clone().query();
        assert!(foo.is_ok());
        assert!(foo.unwrap().test());
        let foo2: Result<Box<dyn Foo2>, _> = x.clone().query();
        assert!(foo2.is_err());
        let bar: Result<Box<Bar>, _> = x.clone().query();
        assert!(bar.is_ok());
    }

    trait Dynamic {
        fn test(&self) -> u32;
    }

    impl Dynamic for Bar {
        fn test(&self) -> u32 {
            42
        }
    }

    #[test]
    fn test_dynamic() {
        let x = Box::new(Bar) as Box<dyn super::Object>;
        let dyn1: Option<&dyn Dynamic> = x.query_ref();
        assert!(dyn1.is_none());

        dynamic_interfaces! {
            Bar: dyn Dynamic;
        }

        let dyn2: Option<&dyn Dynamic> = x.query_ref();
        assert!(dyn2.unwrap().test() == 42);
    }

    #[test]
    fn test_primitives() {
        Box::new(1);
        Box::new(1f32);
        Box::new("test".to_string());
        Box::new(vec![1, 2, 3]);
    }
}
