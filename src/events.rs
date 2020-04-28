//! A basic event handling framework.

use crate::{OwnedResources, ResourcesEnum, ResourcesProvider, World};
use erasable::{erase, Erasable, ErasedPtr};
use fxhash::FxHashMap;
use smallvec::SmallVec;
use std::any::TypeId;
use std::ptr::NonNull;

/// Marker trait for types which can be used as events.
pub trait Event: 'static {}
impl<T> Event for T where T: 'static {}

/// A raw event handler. Use the `event_handler` proc macro
/// instead of implementing this type manually.
#[doc(hidden)]
pub trait RawEventHandler: Send + Sync + 'static {
    type Event: Event;
    fn handle(&self, resources: &ResourcesEnum, world: &mut World, event: &Self::Event);
    fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World);
}

trait TypeErasedEventHandler: Send + Sync + 'static {
    unsafe fn handle(&self, resources: &ResourcesEnum, world: &mut World, event: ErasedPtr);
    fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World);

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<H, E> TypeErasedEventHandler for H
where
    H: RawEventHandler<Event = E>,
    E: Event,
{
    /// Safety: the type of `event` must be the same
    /// as the event type handled by this handler.
    unsafe fn handle(&self, resources: &ResourcesEnum, world: &mut World, event: ErasedPtr) {
        <Self as RawEventHandler>::handle(self, resources, world, E::unerase(event).as_ref())
    }

    fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World) {
        <Self as RawEventHandler>::set_up(self, resources, world);
    }
}

type HandlerVec = SmallVec<[Box<dyn TypeErasedEventHandler>; 4]>;

/// Stores event handlers and allows triggering events.
#[derive(Default)]
pub struct EventHandlers(FxHashMap<TypeId, HandlerVec>);

impl EventHandlers {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an event handler.
    pub fn add<E>(&mut self, handler: impl RawEventHandler<Event = E>)
    where
        E: Event,
    {
        self.0
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(handler))
    }

    /// Builder function to add an event handler.
    pub fn with<E>(mut self, handler: impl RawEventHandler<Event = E>) -> Self
    where
        E: Event,
    {
        self.add(handler);
        self
    }

    pub fn set_up(&mut self, resources: &mut OwnedResources, world: &mut World) {
        for handler in self.0.values_mut().flatten() {
            handler.set_up(resources, world);
        }
    }

    /// Triggers an event.
    pub fn trigger<E>(&self, resources: &impl ResourcesProvider, world: &mut World, event: E)
    where
        E: Event,
    {
        let mut event = event;
        if let Some(handlers) = self.0.get(&TypeId::of::<E>()) {
            for handler in handlers {
                // Safety: we know that the type of `event` is the same type
                // handled by this handler since it's in the handlers vec
                // for that event type ID.
                unsafe {
                    handler.handle(
                        &resources.as_resources_ref(),
                        world,
                        erase(NonNull::new_unchecked((&mut event) as *mut E)),
                    );
                }
            }
        }
    }
}
