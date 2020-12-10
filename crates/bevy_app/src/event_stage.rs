use super::event::{EventReader, Events};
use bevy_ecs::{Stage, World, Resources, System, IntoSystem, Local, Res, ShouldRun, SystemStage, IntoChainSystem};
use std::marker::PhantomData;


pub struct EventStage<T> {
    inner: SystemStage,
    _marker: PhantomData<T>
}

impl<T> Default for EventStage<T>
    where
        T: Clone + Send + Sync + 'static
{
    fn default() -> Self {
        Self::parallel()
    }
}

impl<T> EventStage<T>
    where
        T: Clone + Send + Sync + 'static
{
    pub fn new(system_stage: SystemStage) -> Self {
        let inner = system_stage
            .with_run_criteria(event_stage_run_criteria::<T>);

        Self {
            inner,
            _marker: PhantomData
        }
    }

    pub fn serial() -> Self {
        Self::new(SystemStage::serial())
    }

    pub fn parallel() -> Self {
        Self::new(SystemStage::parallel())
    }

    pub fn with_system<S, Params, IntoS>(mut self, system: IntoS) -> Self
        where
            S: System<Input = T, Output = ()>,
            IntoS: IntoSystem<Params, S>,
    {
        self.inner.add_system_boxed(Box::new(next_event_system.chain(system)));
        self
    }

    pub fn add_system<S, Params, IntoS>(&mut self, system: IntoS) -> &mut Self
        where
            S: System<Input = T, Output = ()>,
            IntoS: IntoSystem<Params, S>,
    {
        self.inner.add_system_boxed(Box::new(next_event_system.chain(system)));
        self
    }
}

impl<T> Stage for EventStage<T>
    where
        T: Clone + Send + Sync + 'static
{
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        self.inner.run(world, resources)
    }
}

/// Execute systems if there exists an event to consume.
fn event_stage_run_criteria<T: Send + Sync + 'static>(
    mut reader: Local<EventReader<T>>,
    events: Res<Events<T>>
) -> ShouldRun {
    if reader.earliest(&events).is_some() {
        ShouldRun::YesAndLoop
    } else {
        ShouldRun::No
    }
}

/// Fetch the next event and return it. This system is chained into all systems added to EventStage.
///
/// Unwrap is okay here because this system will only be run if there exists an event to consume
fn next_event_system<T: Clone + Send + Sync + 'static>(
    mut reader: Local<EventReader<T>>,
    events: Res<Events<T>>
) -> T {
    reader.earliest(&events).unwrap().clone()
}
