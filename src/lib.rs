use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

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

    pub fn fetch<C: 'static>(&self) -> impl Iterator<Item = &C> {
        self.component_store.component_iterator::<C>()
    }

    pub fn fetch_mut<C: 'static>(&mut self) -> impl Iterator<Item = &mut C> {
        self.component_store.component_iterator_mut::<C>()
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

    pub fn component_iterator<'a, C: 'static>(&'a self) -> Box<dyn Iterator<Item = &'a C> + 'a> {
        if !self.components_vecs.contains_key(&TypeId::of::<C>()) {
            return Box::new(std::iter::empty::<&'a C>());
        }

        Box::new(
            self.components_vecs
                .get(&TypeId::of::<C>())
                .unwrap()
                .iter()
                .filter(|c| c.is_some())
                .map(|c| {
                    c.as_ref()
                        .unwrap()
                        .downcast_ref()
                        .expect("Downcasting component into the wrong type")
                }),
        )
    }

    pub fn component_iterator_mut<'a, C: 'static>(
        &'a mut self,
    ) -> Box<dyn Iterator<Item = &'a mut C> + 'a> {
        if !self.components_vecs.contains_key(&TypeId::of::<C>()) {
            return Box::new(std::iter::empty::<&'a mut C>());
        }

        Box::new(
            self.components_vecs
                .get_mut(&TypeId::of::<C>())
                .unwrap()
                .iter_mut()
                .filter(|c| c.is_some())
                .map(|c| {
                    c.as_mut()
                        .unwrap()
                        .downcast_mut()
                        .expect("Downcasting component into the wrong type")
                }),
        )
    }
}

pub struct ReadAccessor<C: 'static>(PhantomData<C>);
pub struct WriteAccessor<C: 'static>(PhantomData<C>);

pub trait Queryable<'a> {
    type Iterator: Iterator + 'a;

    fn query(ecs: &'a mut Ecs) -> Self::Iterator;
}

impl<'a, C: 'static> Queryable<'a> for ReadAccessor<C> {
    type Iterator = Box<dyn Iterator<Item = &'a C> + 'a>;

    fn query(ecs: &'a mut Ecs) -> Self::Iterator {
        Box::new(ecs.fetch::<C>())
    }
}

impl<'a, C: 'static> Queryable<'a> for WriteAccessor<C> {
    type Iterator = Box<dyn Iterator<Item = &'a mut C> + 'a>;

    fn query(ecs: &'a mut Ecs) -> Self::Iterator {
        Box::new(ecs.fetch_mut::<C>())
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
    fn ecs_can_fetch_single_component_immutably() {
        let mut ecs = Ecs::new();
        ecs.create_entity(vec![
            Box::new(Position { x: 6f32, y: 2f32 }),
            Box::new(Velocity { x: 1f32, y: 0f32 }),
        ]);
        ecs.create_entity(vec![Box::new(Position { x: 3f32, y: 24f32 })]);

        assert_eq!(ecs.fetch::<Position>().count(), 2);
        assert_eq!(ecs.fetch::<RectangleShape>().count(), 0);

        let mut positions = ecs.fetch::<Position>();
        assert_eq!(positions.next(), Some(&Position { x: 6f32, y: 2f32 }));
        assert_eq!(positions.next(), Some(&Position { x: 3f32, y: 24f32 }));
    }

    #[test]
    fn ecs_can_fetch_single_component_mutably() {
        let mut ecs = Ecs::new();
        ecs.create_entity(vec![
            Box::new(Position { x: 6f32, y: 2f32 }),
            Box::new(Velocity { x: 1f32, y: 0f32 }),
        ]);
        ecs.create_entity(vec![Box::new(Position { x: 3f32, y: 24f32 })]);

        assert_eq!(ecs.fetch_mut::<Position>().count(), 2);
        assert_eq!(ecs.fetch_mut::<RectangleShape>().count(), 0);

        let mut positions = ecs.fetch_mut::<Position>();
        assert_eq!(positions.next(), Some(&mut Position { x: 6f32, y: 2f32 }));
        assert_eq!(positions.next(), Some(&mut Position { x: 3f32, y: 24f32 }));
    }

    #[test]
    fn read_accessor_query() {
        let mut ecs = Ecs::new();
        ecs.create_entity(vec![
            Box::new(Position { x: 6f32, y: 2f32 }),
            Box::new(Velocity { x: 1f32, y: 0f32 }),
        ]);
        ecs.create_entity(vec![Box::new(Position { x: 3f32, y: 24f32 })]);

        assert_eq!(<ReadAccessor<Position>>::query(&mut ecs).count(), 2);

        let mut positions = <ReadAccessor<Position>>::query(&mut ecs);
        assert_eq!(positions.next(), Some(&Position { x: 6f32, y: 2f32 }));
        assert_eq!(positions.next(), Some(&Position { x: 3f32, y: 24f32 }));
    }

    #[test]
    fn write_accessor_query() {
        let mut ecs = Ecs::new();
        ecs.create_entity(vec![
            Box::new(Position { x: 6f32, y: 2f32 }),
            Box::new(Velocity { x: 1f32, y: 0f32 }),
        ]);
        ecs.create_entity(vec![Box::new(Position { x: 3f32, y: 24f32 })]);

        assert_eq!(<WriteAccessor<Position>>::query(&mut ecs).count(), 2);

        let mut positions = <WriteAccessor<Position>>::query(&mut ecs);
        assert_eq!(positions.next(), Some(&mut Position { x: 6f32, y: 2f32 }));
        assert_eq!(positions.next(), Some(&mut Position { x: 3f32, y: 24f32 }));
    }
}
