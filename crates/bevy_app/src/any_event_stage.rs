use super::event::{EventReader, Events};
use bevy_ecs::{Stage, World, Resources, System, IntoSystem, Local, Res, ShouldRun, SystemStage, IntoChainSystem};
use std::marker::PhantomData;

pub struct AnyEventStage<T> {
    inner: SystemStage,
    _marker: PhantomData<T>
}

impl<A, B, C> Default for AnyEventStage<(A, B, C)>
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        C: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::parallel()
    }
}

impl<A, B, C> AnyEventStage<(A, B, C)>
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        C: Clone + Send + Sync + 'static,
{
    pub fn new(system_stage: SystemStage) -> Self {
        let inner = system_stage
            .with_run_criteria(any_event_stage_run_criteria::<A, B, C>);

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
            S: System<Input = (Option<A>, Option<B>, Option<C>), Output = ()>,
            IntoS: IntoSystem<Params, S>,
    {
        self.inner.add_system_boxed(Box::new(any_event_system.chain(system)));
        self
    }

    pub fn add_system<S, Params, IntoS>(&mut self, system: IntoS) -> &mut Self
        where
            S: System<Input = (Option<A>, Option<B>, Option<C>), Output = ()>,
            IntoS: IntoSystem<Params, S>,
    {
        self.inner.add_system_boxed(Box::new(any_event_system.chain(system)));
        self
    }
}

impl<A, B, C> Stage for AnyEventStage<(A, B, C)>
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        C: Clone + Send + Sync + 'static,
{
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        self.inner.run(world, resources)
    }
}

/// Execute systems if there exists an event to consume.
fn any_event_stage_run_criteria<A, B, C>(
    mut reader_a: Local<EventReader<A>>,
    events_a: Res<Events<A>>,
    mut reader_b: Local<EventReader<B>>,
    events_b: Res<Events<B>>,
    mut reader_c: Local<EventReader<C>>,
    events_c: Res<Events<C>>,
) -> ShouldRun
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        C: Clone + Send + Sync + 'static,
{
    let a = reader_a.earliest(&events_a);
    let b = reader_b.earliest(&events_b);
    let c = reader_c.earliest(&events_c);

    if a.is_some() || b.is_some() || c.is_some() {
        ShouldRun::YesAndLoop
    } else {
        ShouldRun::No
    }
}

fn any_event_system<A, B, C>(
    mut reader_a: Local<EventReader<A>>,
    events_a: Res<Events<A>>,
    mut reader_b: Local<EventReader<B>>,
    events_b: Res<Events<B>>,
    mut reader_c: Local<EventReader<C>>,
    events_c: Res<Events<C>>,
) -> (Option<A>, Option<B>, Option<C>)
    where
        A: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        C: Clone + Send + Sync + 'static,
{
    let a: Option<A> = reader_a.earliest(&events_a).map(|e| e.clone());
    let b: Option<B> = reader_b.earliest(&events_b).map(|e| e.clone());
    let c: Option<C> = reader_c.earliest(&events_c).map(|e| e.clone());

    (a, b, c)
}