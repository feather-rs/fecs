use fecs::{event_handler, system, EntityBuilder, Executor, Resources, SystemCtx, World};

#[test]
fn basic() {
    #[system]
    fn test_system(ctx: &mut SystemCtx) {
        ctx.trigger(10i32);
    }

    struct TestResource(i32);

    #[event_handler]
    fn test_handler(event: &i32, test: &mut TestResource) {
        test.0 = *event;
    }

    let mut executor = Executor::new();
    executor.add_system(test_system);
    executor.add_handler(test_handler);

    let mut resources = Resources::new();
    resources.insert(TestResource(11));

    executor.execute(&resources, &mut World::new());

    assert_eq!(resources.get::<TestResource>().unwrap().0, 10);
}

#[test]
fn queries() {
    #[system]
    fn test_system(world: &mut World) {
        world.query::<&mut i32>().iter().for_each(|mut x| *x += 1);
    }

    let mut executor = Executor::new();
    executor.add_system(test_system);

    let mut world = World::new();
    let entity = EntityBuilder::new()
        .with(15i32)
        .build()
        .spawn_in(&mut world);

    executor.execute(&Resources::default(), &mut world);

    assert_eq!(*world.get::<i32>(entity), 16i32);
}
