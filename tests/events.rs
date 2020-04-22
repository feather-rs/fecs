use fecs::{
    event_handler, Entity, EntityBuilder, EventHandlers, OwnedResources, ResourcesProvider, World,
};

#[test]
fn basic() {
    #[event_handler]
    fn handler(event: &i32, resource: &mut i32, entity: &Entity, world: &mut World) {
        *resource = *event;
        *world.get_mut::<i32>(*entity) = *resource;
    }

    let mut world = World::new();
    let entity = EntityBuilder::new()
        .with(1000i32)
        .build()
        .spawn_in(&mut world);

    let handlers = EventHandlers::new().with(handler);

    let resources = OwnedResources::new().with(entity).with(15i32);

    handlers.trigger(&resources, &mut world, 256i32);

    assert_eq!(*world.get::<i32>(entity), 256);
    assert_eq!(*resources.get::<i32>(), 256);
}
