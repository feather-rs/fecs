use fecs::{EntityBuilder, World};

#[test]
fn build() {
    let mut world = World::new();

    let entity = EntityBuilder::new()
        .with(vec![0i32, 1, 10, 15])
        .with(10u128)
        .with([10usize; 32])
        .build()
        .spawn_in(&mut world);

    let vec = world.get::<Vec<i32>>(entity);
    assert_eq!(vec[0], 0);
    assert_eq!(vec[1], 1);
    assert_eq!(vec[2], 10);
    assert_eq!(vec[3], 15);
    assert_eq!(vec.len(), 4);

    assert_eq!(*world.get::<u128>(entity), 10);
    assert_eq!(*world.get::<[usize; 32]>(entity), [10usize; 32]);
}

#[test]
fn build_one() {
    let mut world = World::new();

    let mut builder = EntityBuilder::new().with(10i32);

    let entity = builder.build_one().spawn_in(&mut world);
    assert_eq!(*world.get::<i32>(entity), 10);

    builder.add(11i32);

    let entity2 = builder.build_one().spawn_in(&mut world);
    assert_eq!(*world.get::<i32>(entity2), 11);

    builder.add(15);

    let entity3 = builder.build_one().spawn_in(&mut world);
    assert_eq!(*world.get::<i32>(entity3), 15);
}
