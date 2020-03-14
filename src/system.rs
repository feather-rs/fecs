use crate::{ErasedVec, World};
use ahash::AHashMap;
use erasable::{Erasable, ErasedPtr};
use object_pool::{Pool, Reusable};
use resources::Resources;
use smallvec::SmallVec;
use std::any::TypeId;
use std::iter;
use std::marker::PhantomData;

pub struct SystemCtx<'a>(Reusable<'a, SystemCtxInner>);

impl<'a> SystemCtx<'a> {
    pub fn trigger<E>(&mut self, event: E)
    where
        E: 'static,
    {
        self.trigger_batched(iter::once(event));
    }

    pub fn trigger_batched<E>(&mut self, events: impl IntoIterator<Item = E>)
    where
        E: 'static,
    {
        self.0.events.extend(events);
    }
}

#[derive(Default)]
pub struct SystemCtxInner {
    events: EventStore,
}

#[doc(hidden)]
pub trait RawSystem: 'static {
    fn run(
        &self,
        resources: &Resources,
        world: &mut World,
        executor: &Executor,
        ctx: &mut SystemCtx,
    );
}

#[doc(hidden)]
pub trait EventHandler<E>: 'static {
    fn handle(
        &self,
        events: &[E],
        resources: &Resources,
        world: &mut World,
        executor: &Executor,
        ctx: &mut SystemCtx,
    );
}

#[doc(hidden)]
pub trait RawEventHandler: 'static {
    unsafe fn handle(
        &self,
        events: ErasedPtr,
        events_len: usize,
        resources: &Resources,
        world: &mut World,
        executor: &Executor,
        ctx: &mut SystemCtx,
    );
    fn event_type(&self) -> TypeId;
}

#[doc(hidden)]
pub struct WrappedEventHandler<E, H> {
    handler: H,
    _phantom: PhantomData<*const E>,
}

impl<E, H> RawEventHandler for WrappedEventHandler<E, H>
where
    E: 'static,
    H: EventHandler<E>,
{
    unsafe fn handle(
        &self,
        events: ErasedPtr,
        events_len: usize,
        resources: &Resources,
        world: &mut World,
        executor: &Executor,
        ctx: &mut SystemCtx,
    ) {
        let events = E::unerase(events);
        let events_slice = std::slice::from_raw_parts(events.as_ptr(), events_len);

        self.handler
            .handle(events_slice, resources, world, executor, ctx);
    }

    fn event_type(&self) -> TypeId {
        TypeId::of::<E>()
    }
}

pub struct Executor {
    systems: Vec<Box<dyn RawSystem>>,
    event_handlers: EventHandlerStore,
    system_ctxs: Pool<'static, SystemCtxInner>,
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            systems: vec![],
            event_handlers: EventHandlerStore::default(),
            system_ctxs: Pool::new(128, || SystemCtxInner::default()),
        }
    }
}

impl Executor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system(&mut self, system: impl RawSystem) {
        self.add_system_boxed(Box::new(system));
    }

    pub fn add_system_boxed(&mut self, system: Box<dyn RawSystem>) {
        self.systems.push(system);
    }

    pub fn add_handler<E>(&mut self, handler: impl EventHandler<E>)
    where
        E: 'static,
    {
        self.event_handlers.insert(handler);
    }

    pub fn add_handler_boxed(&mut self, handler: Box<dyn RawEventHandler>) {
        self.event_handlers
            .insert_raw(handler.event_type(), DynEventHandler(handler));
    }

    pub fn execute(&self, resources: &Resources, world: &mut World) {
        for system in &self.systems {
            let mut ctx = SystemCtx(self.system_ctxs.pull());
            system.run(resources, world, self, &mut ctx);
            self.handle_events(resources, world, &mut ctx);
        }
    }

    pub fn handle_events(&self, resources: &Resources, world: &mut World, ctx: &mut SystemCtx) {
        for (event_type, event_vec) in ctx.0.events.events.iter_mut() {
            if event_vec.len() == 0 {
                continue;
            }

            let event_ptr = event_vec.ptr();
            let event_len = event_vec.len();

            unsafe {
                self.event_handlers.handle(
                    *event_type,
                    event_ptr,
                    event_len,
                    resources,
                    world,
                    self,
                );
            }

            event_vec.clear();
        }
    }
}

struct DynEventHandler(Box<dyn RawEventHandler>);

#[derive(Default)]
struct EventHandlerStore {
    handlers: AHashMap<TypeId, SmallVec<[DynEventHandler; 4]>>,
}

impl EventHandlerStore {
    pub fn insert<E>(&mut self, handler: impl EventHandler<E>)
    where
        E: 'static,
    {
        let wrapped = WrappedEventHandler {
            handler,
            _phantom: PhantomData,
        };
        let dynamic = DynEventHandler(Box::new(wrapped));

        self.insert_raw(TypeId::of::<E>(), dynamic);
    }

    pub fn insert_raw(&mut self, event_type: TypeId, handler: DynEventHandler) {
        self.handlers.entry(event_type).or_default().push(handler);
    }

    pub unsafe fn handle(
        &self,
        event_type: TypeId,
        events: ErasedPtr,
        events_len: usize,
        resources: &Resources,
        world: &mut World,
        executor: &Executor,
    ) {
        if let Some(handlers) = self.handlers.get(&event_type) {
            handlers.iter().for_each(|handler| {
                let mut ctx = SystemCtx(executor.system_ctxs.pull());
                let handler = &handler.0;
                handler.handle(events, events_len, resources, world, executor, &mut ctx);
                executor.handle_events(resources, world, &mut ctx);
            })
        }
    }
}

#[derive(Default)]
pub(crate) struct EventStore {
    pub(crate) events: AHashMap<TypeId, ErasedVec>,
}

impl EventStore {
    pub fn extend<E>(&mut self, events: impl IntoIterator<Item = E>)
    where
        E: 'static,
    {
        unsafe {
            self.events
                .entry(TypeId::of::<E>())
                .or_insert_with(|| ErasedVec::new::<E>())
                .extend(events);
        }
    }
}
