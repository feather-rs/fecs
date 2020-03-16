use crate::{Executor, RawSystem};
use std::sync::Mutex;

pub struct SystemRegistration(pub Mutex<Option<Box<dyn RawSystem>>>);

inventory::collect!(SystemRegistration);

/// Builds an executor using all systems registered with `inventory::submit!`.
///
/// The `system` attribute macro registers systems by default.
///
/// This function may only be called once in the lifecycle of an application.
pub fn build_executor() -> Executor {
    let mut executor = Executor::new();

    for system in inventory::iter::<SystemRegistration> {
        executor.add_boxed(
            system
                .0
                .lock()
                .unwrap()
                .take()
                .expect("already built executor"),
        );
    }
    executor
}
