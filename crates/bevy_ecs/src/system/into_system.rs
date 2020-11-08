use crate::{
    Commands, FetchResource, Query, QuerySet, QueryTuple, ResourceFetchSelf, ResourceQuery,
    Resources, System, SystemId, ThreadLocalExecution,
};
use bevy_hecs::{ArchetypeComponent, Fetch, Query as HecsQuery, QueryAccess, TypeAccess, World};
use std::{any::TypeId, borrow::Cow};

pub struct SystemState {
    id: SystemId,
    name: Cow<'static, str>,
    is_initialized: bool,
    archetype_component_access: TypeAccess<ArchetypeComponent>,
    resource_access: TypeAccess<TypeId>,
    query_archetype_component_accesses: Vec<TypeAccess<ArchetypeComponent>>,
    query_accesses: Vec<Vec<QueryAccess>>,
    query_type_names: Vec<&'static str>,
    commands: Commands,
    current_query_index: usize,
}

impl SystemState {
    pub fn reset_indices(&mut self) {
        self.current_query_index = 0;
    }

    pub fn update(&mut self, world: &World) {
        self.archetype_component_access.clear();
        let mut conflict_index = None;
        let mut conflict_name = None;
        for (i, (query_accesses, component_access)) in self
            .query_accesses
            .iter()
            .zip(self.query_archetype_component_accesses.iter_mut())
            .enumerate()
        {
            component_access.clear();
            for query_access in query_accesses.iter() {
                query_access.get_world_archetype_access(world, Some(component_access));
            }
            if !component_access.is_compatible(&self.archetype_component_access) {
                conflict_index = Some(i);
                conflict_name = component_access
                    .get_conflict(&self.archetype_component_access)
                    .and_then(|archetype_component| {
                        query_accesses
                            .iter()
                            .filter_map(|query_access| {
                                query_access.get_type_name(archetype_component.component)
                            })
                            .next()
                    });
                break;
            }
            self.archetype_component_access.union(component_access);
        }
        if let Some(conflict_index) = conflict_index {
            let mut conflicts_with_index = None;
            for prior_index in 0..conflict_index {
                if !self.query_archetype_component_accesses[conflict_index]
                    .is_compatible(&self.query_archetype_component_accesses[prior_index])
                {
                    conflicts_with_index = Some(prior_index);
                }
            }
            panic!("System {} has conflicting queries. {} conflicts with the component access [{}] in this prior query: {}",
                core::any::type_name::<Self>(),
                self.query_type_names[conflict_index],
                conflict_name.unwrap_or("Unknown"),
                conflicts_with_index.map(|index| self.query_type_names[index]).unwrap_or("Unknown"));
        }
    }
}

pub struct FuncSystem<F, Init, ThreadLocalFunc>
where
    F: FnMut(&mut SystemState, &World, &Resources) + Send + Sync + 'static,
    Init: FnMut(&mut SystemState, &World, &mut Resources) + Send + Sync + 'static,
    ThreadLocalFunc: FnMut(&mut SystemState, &mut World, &mut Resources) + Send + Sync + 'static,
{
    func: F,
    thread_local_func: ThreadLocalFunc,
    init_func: Init,
    state: SystemState,
}

impl<F, Init, ThreadLocalFunc> System for FuncSystem<F, Init, ThreadLocalFunc>
where
    F: FnMut(&mut SystemState, &World, &Resources) + Send + Sync + 'static,
    Init: FnMut(&mut SystemState, &World, &mut Resources) + Send + Sync + 'static,
    ThreadLocalFunc: FnMut(&mut SystemState, &mut World, &mut Resources) + Send + Sync + 'static,
{
    fn name(&self) -> std::borrow::Cow<'static, str> {
        self.state.name.clone()
    }

    fn id(&self) -> SystemId {
        self.state.id
    }

    fn update(&mut self, world: &World) {
        self.state.update(world);
    }

    fn archetype_component_access(&self) -> &TypeAccess<ArchetypeComponent> {
        &self.state.archetype_component_access
    }

    fn resource_access(&self) -> &TypeAccess<std::any::TypeId> {
        &self.state.resource_access
    }

    fn thread_local_execution(&self) -> ThreadLocalExecution {
        ThreadLocalExecution::NextFlush
    }

    fn run(&mut self, world: &World, resources: &Resources) {
        (self.func)(&mut self.state, world, resources)
    }

    fn run_thread_local(&mut self, world: &mut World, resources: &mut Resources) {
        (self.thread_local_func)(&mut self.state, world, resources)
    }

    fn initialize(&mut self, world: &mut World, resources: &mut Resources) {
        (self.init_func)(&mut self.state, world, resources);
        self.state.is_initialized = true;
    }

    fn is_initialized(&self) -> bool {
        self.state.is_initialized
    }
}

pub trait SystemParam {
    fn init(system_state: &mut SystemState, world: &World, resources: &mut Resources);
    fn get_param(system_state: &mut SystemState, world: &World, resources: &Resources) -> Self;
}

impl<'a, Q: HecsQuery> SystemParam for Query<'a, Q> {
    #[inline]
    fn get_param(system_state: &mut SystemState, world: &World, _resources: &Resources) -> Self {
        let query_index = system_state.current_query_index;
        unsafe {
            let world: &'a World = std::mem::transmute(world);
            let archetype_component_access: &'a TypeAccess<ArchetypeComponent> =
                std::mem::transmute(&system_state.query_archetype_component_accesses[query_index]);
            system_state.current_query_index += 1;
            Query::new(world, archetype_component_access)
        }
    }

    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state
            .query_archetype_component_accesses
            .push(TypeAccess::default());
        system_state
            .query_accesses
            .push(vec![<Q::Fetch as Fetch>::access()]);
        system_state
            .query_type_names
            .push(std::any::type_name::<Q>());
    }
}

impl<T: QueryTuple> SystemParam for QuerySet<T> {
    #[inline]
    fn get_param(system_state: &mut SystemState, world: &World, _resources: &Resources) -> Self {
        let query_index = system_state.current_query_index;
        system_state.current_query_index += 1;
        unsafe {
            QuerySet::new(
                world,
                &system_state.query_archetype_component_accesses[query_index],
            )
        }
    }

    fn init(system_state: &mut SystemState, _world: &World, _resources: &mut Resources) {
        system_state
            .query_archetype_component_accesses
            .push(TypeAccess::default());
        system_state.query_accesses.push(T::get_accesses());
        system_state
            .query_type_names
            .push(std::any::type_name::<T>());
    }
}

impl<R> SystemParam for R
where
    R: ResourceQuery + ResourceFetchSelf,
{
    fn init(system_state: &mut SystemState, _world: &World, resources: &mut Resources) {
        R::initialize(resources, Some(system_state.id));
        system_state
            .resource_access
            .union(&<R::Fetch as FetchResource>::access());
    }

    #[inline]
    fn get_param(system_state: &mut SystemState, _world: &World, resources: &Resources) -> Self {
        unsafe { <R as ResourceFetchSelf>::get(resources, Some(system_state.id)) }
    }
}

impl SystemParam for Commands {
    fn init(system_state: &mut SystemState, world: &World, _resources: &mut Resources) {
        system_state
            .commands
            .set_entity_reserver(world.get_entity_reserver())
    }

    #[inline]
    fn get_param(system_state: &mut SystemState, _world: &World, _resources: &Resources) -> Self {
        system_state.commands.clone()
    }
}

pub trait IntoSystem<Params> {
    fn system(self) -> Box<dyn System>;
}

macro_rules! impl_into_system {
    (($($param: ident),*)) => {
        impl<Func, $($param: SystemParam),*> IntoSystem<($($param,)*)> for Func
        where Func: FnMut($($param),*) + Send + Sync + 'static,
        {
            #[allow(unused_variables)]
            fn system(mut self) -> Box<dyn System> {
                Box::new(FuncSystem {
                    state: SystemState {
                        name: std::any::type_name::<Self>().into(),
                        archetype_component_access: TypeAccess::default(),
                        resource_access: TypeAccess::default(),
                        is_initialized: false,
                        id: SystemId::new(),
                        commands: Commands::default(),
                        query_archetype_component_accesses: Vec::new(),
                        query_accesses: Vec::new(),
                        query_type_names: Vec::new(),
                        current_query_index: 0,
                    },
                    func: move |state, world, resources| {
                        state.reset_indices();
                        self($($param::get_param(state, world, resources)),*);
                    },
                    thread_local_func: |state, world, resources| {
                        state.commands.apply(world, resources);
                    },
                    init_func: |state, world, resources| {
                        $($param::init(state, world, resources);)*
                    },
                })
            }
        }

    };
}

impl_into_system!(());
impl_into_system!((A));
impl_into_system!((A, B));
impl_into_system!((A, B, C));
impl_into_system!((A, B, C, D));
impl_into_system!((A, B, C, D, E));
impl_into_system!((A, B, C, D, E, F));
impl_into_system!((A, B, C, D, E, F, G));
impl_into_system!((A, B, C, D, E, F, G, H));
impl_into_system!((A, B, C, D, E, F, G, H, I));
impl_into_system!((A, B, C, D, E, F, G, H, I, J));

#[cfg(test)]
mod tests {
    use super::{IntoSystem, Query};
    use crate::{
        resource::{ResMut, Resources},
        schedule::Schedule,
        ChangedRes, QuerySet,
    };
    use bevy_hecs::{Entity, With, World};

    #[derive(Debug, Eq, PartialEq)]
    struct A;
    struct B;
    struct C;
    struct D;

    #[test]
    fn query_system_gets() {
        fn query_system(
            mut ran: ResMut<bool>,
            entity_query: Query<With<A, Entity>>,
            b_query: Query<&B>,
            a_c_query: Query<(&A, &C)>,
            d_query: Query<&D>,
        ) {
            let entities = entity_query.iter().collect::<Vec<Entity>>();
            assert!(
                b_query.get_component::<B>(entities[0]).is_err(),
                "entity 0 should not have B"
            );
            assert!(
                b_query.get_component::<B>(entities[1]).is_ok(),
                "entity 1 should have B"
            );
            assert!(
                b_query.get_component::<A>(entities[1]).is_err(),
                "entity 1 should have A, but b_query shouldn't have access to it"
            );
            assert!(
                b_query.get_component::<D>(entities[3]).is_err(),
                "entity 3 should have D, but it shouldn't be accessible from b_query"
            );
            assert!(
                b_query.get_component::<C>(entities[2]).is_err(),
                "entity 2 has C, but it shouldn't be accessible from b_query"
            );
            assert!(
                a_c_query.get_component::<C>(entities[2]).is_ok(),
                "entity 2 has C, and it should be accessible from a_c_query"
            );
            assert!(
                a_c_query.get_component::<D>(entities[3]).is_err(),
                "entity 3 should have D, but it shouldn't be accessible from b_query"
            );
            assert!(
                d_query.get_component::<D>(entities[3]).is_ok(),
                "entity 3 should have D"
            );

            *ran = true;
        }

        let mut world = World::default();
        let mut resources = Resources::default();
        resources.insert(false);
        world.spawn((A,));
        world.spawn((A, B));
        world.spawn((A, C));
        world.spawn((A, D));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", query_system.system());

        schedule.run(&mut world, &mut resources);

        assert!(*resources.get::<bool>().unwrap(), "system ran");
    }

    #[test]
    fn or_query_set_system() {
        // Regression test for issue #762
        use crate::{Added, Changed, Mutated, Or};
        fn query_system(
            mut ran: ResMut<bool>,
            set: QuerySet<(
                Query<Or<(Changed<A>, Changed<B>)>>,
                Query<Or<(Added<A>, Added<B>)>>,
                Query<Or<(Mutated<A>, Mutated<B>)>>,
            )>,
        ) {
            let changed = set.q0().iter().count();
            let added = set.q1().iter().count();
            let mutated = set.q2().iter().count();

            assert_eq!(changed, 1);
            assert_eq!(added, 1);
            assert_eq!(mutated, 0);

            *ran = true;
        }

        let mut world = World::default();
        let mut resources = Resources::default();
        resources.insert(false);
        world.spawn((A, B));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", query_system.system());

        schedule.run(&mut world, &mut resources);

        assert!(*resources.get::<bool>().unwrap(), "system ran");
    }

    #[test]
    fn changed_resource_system() {
        fn incr_e_on_flip(_run_on_flip: ChangedRes<bool>, mut query: Query<&mut i32>) {
            for mut i in query.iter_mut() {
                *i += 1;
            }
        }

        let mut world = World::default();
        let mut resources = Resources::default();
        resources.insert(false);
        let ent = world.spawn((0,));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", incr_e_on_flip.system());

        schedule.run(&mut world, &mut resources);
        assert_eq!(*(world.get::<i32>(ent).unwrap()), 1);

        schedule.run(&mut world, &mut resources);
        assert_eq!(*(world.get::<i32>(ent).unwrap()), 1);

        *resources.get_mut::<bool>().unwrap() = true;
        schedule.run(&mut world, &mut resources);
        assert_eq!(*(world.get::<i32>(ent).unwrap()), 2);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_mut_system() {
        fn sys(_q1: Query<&mut A>, _q2: Query<&mut A>) {}

        let mut world = World::default();
        let mut resources = Resources::default();
        world.spawn((A,));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", sys.system());

        schedule.run(&mut world, &mut resources);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_immut_system() {
        fn sys(_q1: Query<&A>, _q2: Query<&mut A>) {}

        let mut world = World::default();
        let mut resources = Resources::default();
        world.spawn((A,));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", sys.system());

        schedule.run(&mut world, &mut resources);
    }

    #[test]
    fn query_set_system() {
        fn sys(_set: QuerySet<(Query<&mut A>, Query<&B>)>) {}

        let mut world = World::default();
        let mut resources = Resources::default();
        world.spawn((A,));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", sys.system());

        schedule.run(&mut world, &mut resources);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_with_query_set_system() {
        fn sys(_query: Query<&mut A>, _set: QuerySet<(Query<&mut A>, Query<&B>)>) {}

        let mut world = World::default();
        let mut resources = Resources::default();
        world.spawn((A,));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", sys.system());

        schedule.run(&mut world, &mut resources);
    }

    #[test]
    #[should_panic]
    fn conflicting_query_sets_system() {
        fn sys(_set_1: QuerySet<(Query<&mut A>,)>, _set_2: QuerySet<(Query<&mut A>, Query<&B>)>) {}

        let mut world = World::default();
        let mut resources = Resources::default();
        world.spawn((A,));

        let mut schedule = Schedule::default();
        schedule.add_stage("update");
        schedule.add_system_to_stage("update", sys.system());

        schedule.run(&mut world, &mut resources);
    }
}
