use fecs::system;

#[system]
fn sys() {}

#[test]
fn registry() {
    assert_eq!(fecs::build_executor().num_systems(), 1);
}
