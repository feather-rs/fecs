use crate::World;
use legion::entity::Entity;
use legion::filter::{ArchetypeFilterData, Filter};
use legion::iterator::SliceVecIter;
use legion::storage::{
    ArchetypeDescription, Component, ComponentMeta, ComponentStorage, ComponentTypeId,
};
use legion::world::{ComponentLayout, ComponentSource, IntoComponentSource};
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[derive(Default)]
pub struct EntityBuilder {
    /// Raw component storage. Each component is written
    /// unaligned into this vector.
    components: Vec<u8>,
    /// Stores the type IDs, meta, and offset into `components`
    /// for each component in this builder.
    component_data: Vec<(ComponentTypeId, ComponentMeta, usize)>,
    /// Index of next byte to write in `components`.
    cursor: usize,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<C>(mut self, component: C) -> Self
    where
        C: Component,
    {
        self.add(component);

        self
    }

    pub fn add<C>(&mut self, component: C) -> &mut Self
    where
        C: Component,
    {
        // If the component already exists in the store,
        // then override it.
        if let Some((ty, meta, offset)) = self
            .component_data
            .iter()
            .find(|(ty, _, _)| *ty == ComponentTypeId::of::<C>())
            .copied()
        {
            debug_assert!(ty == ComponentTypeId::of::<C>());
            debug_assert!(meta == ComponentMeta::of::<C>());
            unsafe { self.replace(component, offset) }
            return self;
        }

        let size = mem::size_of::<C>();
        let required_capacity = self.cursor + size;

        if self.components.capacity() < required_capacity {
            self.components.reserve(required_capacity);
        }
        debug_assert!(self.components.capacity() >= required_capacity);

        unsafe {
            self.components
                .as_mut_ptr()
                .add(self.cursor)
                .cast::<C>()
                .write_unaligned(component);
        }

        let type_id = ComponentTypeId::of::<C>();
        let meta = ComponentMeta::of::<C>();
        self.component_data.push((type_id, meta, self.cursor));

        self.cursor += size;

        self
    }

    unsafe fn replace<C>(&mut self, component: C, offset: usize) {
        self.components
            .as_mut_ptr()
            .add(offset)
            .cast::<C>()
            .write_unaligned(component);
    }

    pub fn build(self) -> BuiltEntity<'static> {
        BuiltEntity {
            builder: CowMut::Owned(self),
            written: false,
        }
    }

    pub fn build_one(&mut self) -> BuiltEntity {
        BuiltEntity {
            builder: CowMut::Borrowed(self),
            written: false,
        }
    }
}

enum CowMut<'a, T> {
    Borrowed(&'a mut T),
    Owned(T),
}

impl<'a, T> Deref for CowMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            CowMut::Borrowed(x) => *x,
            CowMut::Owned(x) => x,
        }
    }
}

impl<'a, T> DerefMut for CowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            CowMut::Borrowed(x) => *x,
            CowMut::Owned(x) => x,
        }
    }
}

pub struct BuiltEntity<'a> {
    builder: CowMut<'a, EntityBuilder>,
    written: bool,
}

impl<'a> IntoComponentSource for BuiltEntity<'a> {
    type Source = Self;

    fn into(self) -> Self::Source {
        self
    }
}

impl<'a> BuiltEntity<'a> {
    pub fn spawn_in(self, world: &mut World) -> Entity {
        world.spawn(self)[0]
    }
}

impl<'a> ComponentSource for BuiltEntity<'a> {
    fn is_empty(&mut self) -> bool {
        self.written
    }

    fn len(&self) -> usize {
        1
    }

    fn write<T>(&mut self, mut allocated: T, chunk: &mut ComponentStorage) -> usize
    where
        T: Iterator<Item = Entity>,
    {
        let mut writer = chunk.writer();

        let (entities, components) = writer.get();
        let components = unsafe { &mut *components.get() };

        entities.push(allocated.next().expect("not enough entities"));

        let builder = self.builder.deref_mut();

        for (type_id, _meta, offset) in &builder.component_data {
            let component_resource_set = components.get_mut(*type_id).expect("invalid archetype");
            let mut component_writer = component_resource_set.writer();

            unsafe {
                let ptr = NonNull::new(builder.components.as_mut_ptr().add(*offset))
                    .expect("ptr is null... this should not happen");

                component_writer.push_raw(ptr, 1);
            }
        }

        builder.component_data.clear();
        builder.components.clear();
        builder.cursor = 0;

        self.written = true;

        1
    }
}

impl<'a> ComponentLayout for BuiltEntity<'a> {
    type Filter = Self;

    fn get_filter(&mut self) -> &mut Self::Filter {
        self
    }

    fn tailor_archetype(&self, archetype: &mut ArchetypeDescription) {
        for (type_id, meta, _) in &self.builder.component_data {
            archetype.register_component_raw(*type_id, *meta);
        }
    }
}

impl<'a, 'b> Filter<ArchetypeFilterData<'b>> for BuiltEntity<'a> {
    type Iter = SliceVecIter<'b, ComponentTypeId>;

    fn collect(&self, source: ArchetypeFilterData<'b>) -> Self::Iter {
        source.component_types.iter()
    }

    fn is_match(&self, item: &<Self::Iter as Iterator>::Item) -> Option<bool> {
        if item.len() != self.builder.component_data.len() {
            return Some(false);
        }

        for (type_id, _, _) in &self.builder.component_data {
            if !item.contains(type_id) {
                return Some(false);
            }
        }

        Some(true)
    }
}
