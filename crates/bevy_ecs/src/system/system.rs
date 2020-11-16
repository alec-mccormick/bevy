use crate::{ArchetypeComponent, Resources, TypeAccess, World};
use std::{any::TypeId, borrow::Cow};

/// Determines the strategy used to run the `run_thread_local` function in a [System]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ThreadLocalExecution {
    Immediate,
    NextFlush,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SystemId(pub usize);

impl SystemId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        SystemId(rand::random::<usize>())
    }
}

/// An ECS system that can be added to a [Schedule](crate::Schedule)
pub trait System<Input = (), Return = ()>: Send + Sync {
    fn name(&self) -> Cow<'static, str>;
    fn id(&self) -> SystemId;
    fn is_initialized(&self) -> bool;
    fn update(&mut self, world: &World);
    fn archetype_component_access(&self) -> &TypeAccess<ArchetypeComponent>;
    fn resource_access(&self) -> &TypeAccess<TypeId>;
    fn thread_local_execution(&self) -> ThreadLocalExecution;
    unsafe fn run_unsafe(&mut self, input: Input, world: &World, resources: &Resources) -> Option<Return>;
    fn run(&mut self, input: Input, world: &mut World, resources: &mut Resources) -> Option<Return> {
        // SAFE: world and resources are exclusively borrowed
        unsafe {
            self.run_unsafe(input, world, resources)
        }
    }
    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources);
    fn initialize(&mut self, _world: &mut World, _resources: &mut Resources) {}
}

pub struct ChainSystem<AIn, AOut, BOut> {
    a: Box<dyn System<AIn, AOut>>,
    b: Box<dyn System<AOut, BOut>>,
    name: Cow<'static, str>,
    id: SystemId,
    pub(crate) archetype_component_access: TypeAccess<ArchetypeComponent>,
    pub(crate) resource_access: TypeAccess<TypeId>,
}

impl<AIn, AOut, BOut>  System<AIn, BOut> for ChainSystem<AIn, AOut, BOut> {
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }

    fn id(&self) -> SystemId {
        self.id
    }

    fn is_initialized(&self) -> bool {
        self.a.is_initialized() && self.b.is_initialized()
    }

    fn update(&mut self, world: &World) {
        self.archetype_component_access.clear();
        self.resource_access.clear();
        self.a.update(world);
        self.b.update(world);

        self.archetype_component_access.union(self.a.archetype_component_access());
        self.resource_access.union(self.b.resource_access());
    }

    fn archetype_component_access(&self) -> &TypeAccess<ArchetypeComponent> {
        &self.archetype_component_access
    }

    fn resource_access(&self) -> &TypeAccess<TypeId> {
        &self.resource_access
    }

    fn thread_local_execution(&self) -> ThreadLocalExecution {
        ThreadLocalExecution::NextFlush
    }

    unsafe fn run_unsafe(&mut self, input: AIn, world: &World, resources: &Resources) -> Option<BOut> {
        let out = self.a.run_unsafe(input, world, resources).unwrap();
        self.b.run_unsafe(out, world, resources)
    }

    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources) {
        self.a.run_thread_local(world, resources);
        self.b.run_thread_local(world, resources);
    }
}

pub trait AsChainSystem<AIn, AOut, BOut> {
    fn chain(self, system: Box<dyn System<AOut, BOut>>) -> Box<dyn System<AIn, BOut>>;
}

impl<AIn: 'static, AOut: 'static, BOut: 'static>  AsChainSystem<AIn, AOut, BOut> for Box<dyn System<AIn, AOut>> {
    fn chain(self, system: Box<dyn System<AOut, BOut>>) -> Box<dyn System<AIn, BOut>> {
        Box::new(ChainSystem {
            name: Cow::Owned(format!("Chain({}, {})", self.name(), system.name())),
            a: self,
            b: system,
            archetype_component_access: Default::default(),
            resource_access: Default::default(),
            id: SystemId::new(),
        })
    }
}

pub struct FilledInputSystem<Input: Clone, Output> {
    system: Box<dyn System<Input, Output>>,
    input: Input,
}

impl<Input: Clone + Send + Sync, Output> System<(), Output> for FilledInputSystem<Input, Output> {
    fn name(&self) -> Cow<'static, str> {
        self.system.name()
    }

    fn id(&self) -> SystemId {
        self.system.id()
    }

    fn is_initialized(&self) -> bool {
        self.system.is_initialized()
    }

    fn update(&mut self, world: &World) {
        self.system.update(world);
    }

    fn archetype_component_access(&self) -> &TypeAccess<ArchetypeComponent> {
        self.system.archetype_component_access()
    }

    fn resource_access(&self) -> &TypeAccess<TypeId> {
        self.system.resource_access()
    }

    fn thread_local_execution(&self) -> ThreadLocalExecution {
        self.system.thread_local_execution()
    }

    unsafe fn run_unsafe(&mut self, _input: (), world: &World, resources: &Resources) -> Option<Output> {
        self.system.run_unsafe(self.input.clone(), world, resources)
    }

    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources) {
        self.system.run_thread_local(world, resources);
    }
}

pub trait FillSystemInput<Input, Output> {
    fn input(self, input: Input) -> Box<dyn System<(), Output>>;
}

impl<Input: Clone + Send + Sync + 'static, Output: 'static> FillSystemInput<Input, Output> for Box<dyn System<Input, Output>> {
    fn input(self, input: Input) -> Box<dyn System<(), Output>> {
        Box::new(FilledInputSystem {
            system: self,
            input,
        })
    }
}