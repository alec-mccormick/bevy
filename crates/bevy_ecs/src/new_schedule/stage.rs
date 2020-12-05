use crate::{IntoSystem, Resources, System, SystemId, World};
use bevy_utils::HashSet;
use downcast_rs::{Downcast, impl_downcast};

use super::{ParallelSystemStageExecutor, SerialSystemStageExecutor, SystemStageExecutor};

pub enum StageError {
    SystemAlreadyExists(SystemId),
}

pub trait Stage: Downcast {
    fn run(&mut self, world: &mut World, resources: &mut Resources);
}

impl_downcast!(Stage);

pub struct SystemStage {
    systems: Vec<Box<dyn System<Input = (), Output = ()>>>,
    system_ids: HashSet<SystemId>,
    executor: Box<dyn SystemStageExecutor>,
    should_run: Option<Box<dyn System<Input = (), Output = bool>>>,
    changed_systems: Vec<usize>,
    intialized_should_run: bool,
}

impl SystemStage {
    pub fn new(executor: Box<dyn SystemStageExecutor>) -> Self {
        SystemStage {
            executor,
            intialized_should_run: false,
            systems: Default::default(),
            system_ids: Default::default(),
            should_run: Default::default(),
            changed_systems: Default::default(),
        }
    }

    pub fn serial() -> Self {
        Self::new(Box::new(SerialSystemStageExecutor::default()))
    }

    pub fn parallel() -> Self {
        Self::new(Box::new(ParallelSystemStageExecutor::default()))
    }

    pub fn add_system<S, Params, IntoS>(&mut self, system: IntoS) -> &mut Self
    where
        S: System<Input = (), Output = ()>,
        IntoS: IntoSystem<Params, S>,
    {
        self.add_system_boxed(Box::new(system.system()));
        self
    }

    pub fn add_system_boxed(
        &mut self,
        system: Box<dyn System<Input = (), Output = ()>>,
    ) -> &mut Self {
        if self.system_ids.contains(&system.id()) {
            panic!(
                "System with id {:?} ({}) already exists",
                system.id(),
                system.name()
            );
        }
        self.system_ids.insert(system.id());
        self.changed_systems.push(self.systems.len());
        self.systems.push(system);
        self
    }

    pub fn get_executor<T: SystemStageExecutor>(&self) -> Option<&T> {
        self.executor.downcast_ref()
    } 

    pub fn get_executor_mut<T: SystemStageExecutor>(&mut self) -> Option<&mut T> {
        self.executor.downcast_mut()
    } 
}

impl Stage for SystemStage {
    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        if !self.intialized_should_run {
            if let Some(should_run) = &mut self.should_run {
                should_run.initialize(world, resources)
            }
            self.intialized_should_run = true;
        }

        let changed_systems = std::mem::take(&mut self.changed_systems);
        for system_index in changed_systems.iter() {
            self.systems[*system_index].initialize(world, resources);
        }
        self.executor
            .execute_stage(&mut self.systems, &changed_systems, world, resources);
    }
}
