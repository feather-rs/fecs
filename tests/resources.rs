use fecs::{OwnedResources, RefResources, ResourcesProvider};

#[test]
fn resources() {
    let mut resources = OwnedResources::new();

    resources.insert(10i32);

    assert_eq!(*resources.get::<i32>(), 10);
    assert!(resources.try_get::<i64>().is_none());

    resources.insert(11i64);
    assert_eq!(*resources.get::<i64>(), 11);
}

#[test]
#[should_panic]
fn borrow_mutable_twice() {
    let mut resources = OwnedResources::new();

    resources.insert(10i32);

    let _ref = resources.get_mut::<i32>();
    resources.get_mut::<i32>();
}

#[test]
#[should_panic]
fn borrow_immutable_and_mutable() {
    let mut resources = OwnedResources::new();

    resources.insert(10i32);

    let _ref = resources.get::<i32>();
    resources.get_mut::<i32>();
}

#[test]
fn refs() {
    let resources = OwnedResources::new().with(10i32).with(15u64);

    let mut r = "bla";
    let resources = RefResources::new(&resources, (&mut r,));

    assert_eq!(*resources.get::<i32>(), 10);
    assert_eq!(*resources.get::<u64>(), 15);
    assert_eq!(*resources.get::<&'static str>(), "bla");
    *resources.get_mut::<&'static str>() = "test";
    assert_eq!(*resources.get::<&'static str>(), "test");

    drop(resources);
    assert_eq!(r, "test");
}
