use core::marker::PhantomData;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

pub type EntityId = usize;
pub type Component = Box<dyn Any>;
pub type ComponentType = TypeId;

#[derive(Debug)]
pub struct Ecs {
    component_store: ComponentStore,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            component_store: ComponentStore::new(),
        }
    }

    pub fn create_entity(&mut self, components: Vec<Component>) -> EntityId {
        self.component_store
            .allocate_entity_with_components(components)
    }

    pub fn get<C: 'static + Debug>(&self, entity_id: EntityId) -> Option<&C> {
        self.component_store.get_component::<C>(entity_id)
    }
}

#[derive(Debug)]
struct ComponentStore {
    components_vecs: HashMap<TypeId, Vec<Option<Component>>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        ComponentStore {
            components_vecs: HashMap::new(),
        }
    }

    pub fn allocate_entity_with_components(&mut self, components: Vec<Component>) -> EntityId {
        for component in components {
            let type_id = (*component).type_id();
            self.components_vecs
                .entry(type_id)
                .or_insert(vec![])
                .push(Some(component));
        }
        self.components_vecs.iter().next().unwrap().1.len() - 1
    }

    pub fn get_component<C: 'static + Debug>(&self, entity_id: EntityId) -> Option<&C> {
        self.components_vecs
            .get(&TypeId::of::<C>())?
            .get(entity_id)?
            .as_ref()?
            .downcast_ref()
    }
}

pub struct ReadAccessor<C> {
    marker: PhantomData<C>,
}

pub struct WriteAccessor<C> {
    marker: PhantomData<C>,
}

trait Accessor {
    fn query_description() -> QueryDescription;
}

trait Query {
    fn query_description() -> QueryDescription;
}

pub struct QueryDescription {
    pub read_components: Vec<ComponentType>,
    pub written_components: Vec<ComponentType>,
}

impl<C: 'static> Accessor for ReadAccessor<C> {
    fn query_description() -> QueryDescription {
        QueryDescription {
            read_components: vec![TypeId::of::<C>()],
            written_components: vec![],
        }
    }
}

impl<C: 'static> Accessor for WriteAccessor<C> {
    fn query_description() -> QueryDescription {
        QueryDescription {
            read_components: vec![],
            written_components: vec![TypeId::of::<C>()],
        }
    }
}

macro_rules! impl_query_tuple {
    ($H:ident$(, $T:tt)+) => {
        impl<$H: Accessor $(, $T: Accessor)+> Query for ($H $(,$T)*,) {
            fn query_description() -> QueryDescription {
                let mut description = QueryDescription {
                    read_components: vec![],
                    written_components: vec![]
                };

                let mut h_description = $H::query_description();
                description.read_components.append(&mut h_description.read_components);
                description.written_components.append(&mut h_description.written_components);


                $({
                    let mut t_description = $T::query_description();
                    description.read_components.append(&mut t_description.read_components);
                    description.written_components.append(&mut t_description.written_components);
                })+
                description
            }
        }
    }
}

impl_query_tuple!(A, B);
impl_query_tuple!(A, B, C);
impl_query_tuple!(A, B, C, D);
impl_query_tuple!(A, B, C, D, E);
impl_query_tuple!(A, B, C, D, E, F);
impl_query_tuple!(A, B, C, D, E, F, G);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Velocity {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct RectangleShape {
        pub width: f32,
        pub height: f32,
    }

    #[test]
    fn ecs_should_create_entity() {
        let mut ecs = Ecs::new();
        let entity_id = ecs.create_entity(vec![
            Box::new(Position { x: 5f32, y: 2f32 }),
            Box::new(Velocity { x: 1f32, y: 0f32 }),
        ]);

        assert_eq!(
            Position { x: 5f32, y: 2f32 },
            *ecs.get::<Position>(entity_id).unwrap()
        );
        assert_eq!(
            Velocity { x: 1f32, y: 0f32 },
            *ecs.get::<Velocity>(entity_id).unwrap()
        );
        assert_eq!(None, ecs.get::<RectangleShape>(entity_id));
    }

    #[test]
    fn query_with_single_read_accessor_can_be_transformed_into_query_description() {
        let query_description = <ReadAccessor<Position>>::query_description();

        assert_eq!(1, query_description.read_components.len());
        assert_eq!(0, query_description.written_components.len());
        assert_eq!(
            &TypeId::of::<Position>(),
            query_description.read_components.get(0).unwrap()
        );
    }

    #[test]
    fn query_with_single_write_accessor_can_be_transformed_into_query_description() {
        let query_description = <WriteAccessor<Position>>::query_description();

        assert_eq!(0, query_description.read_components.len());
        assert_eq!(1, query_description.written_components.len());
        assert_eq!(
            &TypeId::of::<Position>(),
            query_description.written_components.get(0).unwrap()
        )
    }

    #[test]
    fn query_with_multiple_read_accessor_can_be_transformed_into_query_description() {
        let query_description =
            <(ReadAccessor<Position>, ReadAccessor<Velocity>)>::query_description();

        assert_eq!(2, query_description.read_components.len());
        assert_eq!(0, query_description.written_components.len());
        assert_eq!(
            &TypeId::of::<Position>(),
            query_description.read_components.get(0).unwrap()
        );
        assert_eq!(
            &TypeId::of::<Velocity>(),
            query_description.read_components.get(1).unwrap()
        );
    }

    #[test]
    fn query_with_mixed_accessors_can_be_transformed_into_query_description() {
        let query_description = <(
            WriteAccessor<Position>,
            ReadAccessor<Velocity>,
            ReadAccessor<RectangleShape>,
        )>::query_description();

        assert_eq!(2, query_description.read_components.len());
        assert_eq!(1, query_description.written_components.len());
        assert_eq!(
            &TypeId::of::<Velocity>(),
            query_description.read_components.get(0).unwrap()
        );
        assert_eq!(
            &TypeId::of::<RectangleShape>(),
            query_description.read_components.get(1).unwrap()
        );
        assert_eq!(
            &TypeId::of::<Position>(),
            query_description.written_components.get(0).unwrap()
        )
    }
}
