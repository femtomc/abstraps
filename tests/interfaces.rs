use abstraps::*;
use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Foo3;
interfaces!(Foo3: dyn ObjectClone, dyn Debug, dyn Bar2<Foo3>);

trait Bar2<T> {
    fn do_something(&self) -> Option<T>;
}

impl Bar2<Foo3> for Foo3 {
    fn do_something(&self) -> Option<Foo3> {
        println!("I'm a Foo3!");
        None
    }
}

trait SomeOther {
    fn do_other(&self);
}

impl SomeOther for Foo3 {
    fn do_other(&self) {}
}

#[test]
fn test_dynamic_bad() {
    dynamic_interfaces! {
        Foo3: dyn SomeOther;
    }
    let obj = Box::new(Foo3) as Box<dyn Object>;
    let r: Option<&dyn SomeOther> = obj.query_ref();
    r.unwrap().do_other();
}
