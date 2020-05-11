use crate::resources::ResourcesEnum;
use crate::{OwnedResources, ResourcesProvider, World};

#[doc(hidden)]
pub trait RawSystem: Send + Sync + 'static {
    /// Runs the system with the given resources and world.
    fn run(&self, resources: &ResourcesEnum, world: &mut World, executor: &Executor);

    /// Set up the system with the given resources and world.
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

    /// Adds the given system to the executor.
    pub fn add(&mut self, system: impl RawSystem) {
        self.add_boxed(Box::new(system));
    }

    /// Adds the given system to the exectuor.
    pub fn add_boxed(&mut self, system: Box<dyn RawSystem>) {
        self.systems.push(system);
    }

    /// Adds the given system to the executor.
    ///
    /// Returns `Self` such that method calls for `Executor` can be chained.
    pub fn with(mut self, system: impl RawSystem) -> Self {
        self.add(system);
        self
    }

    /// Returns the number of system registrede for this executor.
    pub fn num_systems(&self) -> usize {
        self.systems.len()
    }

    /// Setsup each system registred for this executor.
    ///
    /// # Note
    /// This function should only be called once.
    pub fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World) {
        for system in &mut self.systems {
            system.set_up(resources, world);
        }
    }

    /// Executes the systems in series.
    pub fn execute(&self, resources: &impl ResourcesProvider, world: &mut World) {
        for system in &self.systems {
            system.run(&resources.as_resources_ref(), world, self);
        }
    }
}

static_assertions::assert_impl_all!(Executor: Send, Sync);
