use fecs::Resources;

#[test]
fn resources() {
    let mut resources = Resources::new();

    resources.insert(10i32);

    assert_eq!(*resources.get::<i32>(), 10);
    assert!(resources.try_get::<i64>().is_none());

    resources.insert(11i64);
    assert_eq!(*resources.get::<i64>(), 11);
}

#[test]
#[should_panic]
fn borrow_mutable_twice() {
    let mut resources = Resources::new();

    resources.insert(10i32);

    let _ref = resources.get_mut::<i32>();
    resources.get_mut::<i32>();
}

#[test]
#[should_panic]
fn borrow_immutable_and_mutable() {
    let mut resources = Resources::new();

    resources.insert(10i32);

    let _ref = resources.get::<i32>();
    resources.get_mut::<i32>();
}
