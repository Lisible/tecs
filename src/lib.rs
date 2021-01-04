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

    pub fn fetch<'a, C: 'static>(&'a self) -> impl Iterator<Item = &'a C> {
        self.component_store.component_iterator::<C>()
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

    pub fn component_iterator<'a, C: 'static>(&'a self) -> impl Iterator<Item = &'a C> {
        self.components_vecs
            .get(&TypeId::of::<C>())
            .unwrap()
            .iter()
            .map(|c| c.as_ref().unwrap().downcast_ref().unwrap())
    }
}

pub struct ComponentIterator {}

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
    fn query(ecs: &Ecs);
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
    fn ecs_can_fetch_single_component() {
        let mut ecs = Ecs::new();
        ecs.create_entity(vec![
            Box::new(Position { x: 6f32, y: 2f32 }),
            Box::new(Velocity { x: 1f32, y: 0f32 }),
        ]);
        ecs.create_entity(vec![Box::new(Position { x: 3f32, y: 24f32 })]);

        assert_eq!(ecs.fetch::<Position>().count(), 2);

        let mut positions = ecs.fetch::<Position>();
        assert_eq!(positions.next(), Some(&Position { x: 6f32, y: 2f32 }));
        assert_eq!(positions.next(), Some(&Position { x: 3f32, y: 24f32 }));
    }
}
