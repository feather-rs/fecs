use fecs::{system, EntityBuilder, Executor, OwnedResources, ResourcesProvider, World};

#[test]
fn basic() {
    #[system]
    fn test_system(res: &mut i32) {
        *res += 1024;
    }

    let mut executor = Executor::new();
    executor.add(test_system);

    let mut resources = OwnedResources::new();
    resources.insert(1024i32);

    executor.execute(&resources, &mut World::new());

    assert_eq!(*resources.get::<i32>(), 2048);
}

#[test]
fn queries() {
    #[system]
    fn test_system(world: &mut World) {
        world
            .query::<&mut i32>()
            .iter_mut()
            .for_each(|mut x| *x += 1);
    }

    let mut executor = Executor::new();
    executor.add(test_system);

    let mut world = World::new();
    let entity = EntityBuilder::new()
        .with(15i32)
        .build()
        .spawn_in(&mut world);

    executor.execute(&OwnedResources::default(), &mut world);

    assert_eq!(*world.get::<i32>(entity), 16i32);
}
