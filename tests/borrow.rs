use fecs::AtomicRefCell;

#[test]
fn borrow() {
    let val = AtomicRefCell::new(10);

    assert_eq!(*val.borrow(), 10);
    let guard1 = val.borrow();
    let guard2 = val.borrow();

    assert_eq!(*guard1, *guard2);
    assert!(val.try_borrow_mut().is_err());

    drop(guard1);
    assert!(val.try_borrow_mut().is_err());
    drop(guard2);

    let mut guard2 = val.borrow_mut();
    *guard2 += 1;

    assert_eq!(*guard2, 11);
    drop(guard2);
    assert_eq!(*val.borrow(), 11);
}
