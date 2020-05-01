use crate::resources::ResourcesEnum;
use crate::{OwnedResources, ResourcesProvider, World};

#[doc(hidden)]
pub trait RawSystem: Send + Sync + 'static {
    fn run(&self, resources: &ResourcesEnum, world: &mut World, executor: &Executor);
    fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World);
}

pub struct Executor {
    systems: Vec<Box<dyn RawSystem>>,
}

impl Default for Executor {
    fn default() -> Self {
        Self { systems: vec![] }
    }
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, system: impl RawSystem) {
        self.add_boxed(Box::new(system));
    }

    pub fn add_boxed(&mut self, system: Box<dyn RawSystem>) {
        self.systems.push(system);
    }

    pub fn with(mut self, system: impl RawSystem) -> Self {
        self.add(system);
        self
    }

    pub fn num_systems(&self) -> usize {
        self.systems.len()
    }

    pub fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World) {
        for system in &mut self.systems {
            system.set_up(resources, world);
        }
    }

    pub fn execute(&self, resources: &impl ResourcesProvider, world: &mut World) {
        for system in &self.systems {
            system.run(&resources.as_resources_ref(), world, self);
        }
    }
}

static_assertions::assert_impl_all!(Executor: Send, Sync);
