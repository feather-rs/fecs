use fecs::{system, EventHandler, Executor, Resources, SystemCtx, World};

#[system]
fn test_system(ctx: &mut SystemCtx) {
    ctx.trigger(10i32);
}

struct TestResource(i32);

struct TestHandler;

impl EventHandler<i32> for TestHandler {
    fn handle(
        &self,
        events: &[i32],
        resources: &Resources,
        _world: &mut World,
        _executor: &Executor,
        _ctx: &mut SystemCtx,
    ) {
        resources.get_mut::<TestResource>().unwrap().0 = events[0];
        assert_eq!(events.len(), 1);
    }
}

#[test]
fn basic() {
    let mut executor = Executor::new();
    executor.add_system(test_system);
    executor.add_handler(TestHandler);

    let mut resources = Resources::new();
    resources.insert(TestResource(11));

    executor.execute(&resources, &mut World::new());

    assert_eq!(resources.get::<TestResource>().unwrap().0, 10);
}
