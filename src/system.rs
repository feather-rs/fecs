use crate::resources::ResourcesEnum;
use crate::{ResourcesProvider, World};

#[doc(hidden)]
pub trait RawSystem: 'static {
    fn run(&self, resources: &ResourcesEnum, world: &mut World, executor: &Executor);
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

    pub fn execute(&self, resources: &impl ResourcesProvider, world: &mut World) {
        for system in &self.systems {
            system.run(&resources.as_resources_ref(), world, self);
        }
    }
}
