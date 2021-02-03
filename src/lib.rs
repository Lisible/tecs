use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type EntityId = usize;
pub type Component = Box<dyn Any>;
pub type ComponentType = TypeId;

pub struct Ecs {
    archetypes: HashMap<Box<[ComponentType]>, Archetype>,
    entity_store: EntityStore,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            archetypes: HashMap::new(),
            entity_store: EntityStore::new(),
        }
    }

    pub fn create_entity<D: ComponentsDefinition>(&mut self, _components_definition: D) {
        let _archetype = self.get_or_insert_archetype::<D>();
    }

    pub fn entity_count(&self) -> usize {
        self.entity_store.entity_count()
    }

    pub fn archetype<D: ComponentsDefinition>(&self) -> Option<&Archetype> {
        self.archetypes.get(&D::component_types())
    }

    fn get_or_insert_archetype<D: ComponentsDefinition>(&mut self) -> &mut Archetype {
        self.archetypes
            .entry(D::component_types())
            .or_insert(Archetype::new())
    }
}

pub struct EntityStore {
    next_id: EntityId,
    free_list: Vec<EntityId>,
}

impl EntityStore {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            free_list: vec![],
        }
    }

    pub fn allocate_entity(&mut self) -> EntityId {
        let id = if self.free_list.is_empty() {
            let next_id = self.next_id;
            self.next_id += 1;
            next_id
        } else {
            self.free_list.pop().unwrap()
        };
        id
    }

    pub fn free_entity(&mut self, id: EntityId) {
        assert!(id < self.next_id);
        self.free_list.push(id);
    }

    pub fn entity_count(&self) -> usize {
        self.next_id - self.free_list.len() - 1
    }
}

pub struct Archetype;

impl Archetype {
    pub fn new() -> Self {
        Self
    }
}

pub trait ComponentsDefinition {
    fn component_types() -> Box<[ComponentType]>;
}

impl<A: 'static, B: 'static> ComponentsDefinition for (A, B) {
    fn component_types() -> Box<[ComponentType]> {
        Box::new([TypeId::of::<A>(), TypeId::of::<B>()])
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
    pub fn ecs_new() {
        let ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
    }

    #[test]
    pub fn entity_store_new() {
        let entity_store = EntityStore::new();
        assert_eq!(entity_store.entity_count(), 0);
    }

    #[test]
    pub fn entity_store_allocate() {
        let mut entity_store = EntityStore::new();
        let first_entity_id = entity_store.allocate_entity();
        assert_eq!(entity_store.entity_count(), 1);
        assert_eq!(first_entity_id, 1);
    }

    #[test]
    pub fn entity_store_reallocate() {
        let mut entity_store = EntityStore::new();
        let first_entity_id = entity_store.allocate_entity();
        assert_eq!(entity_store.entity_count(), 1);
        assert_eq!(first_entity_id, 1);

        entity_store.free_entity(first_entity_id);
        assert_eq!(entity_store.entity_count(), 0);

        let second_entity_id = entity_store.allocate_entity();
        assert_eq!(entity_store.entity_count(), 1);
        assert_eq!(second_entity_id, 1)
    }
}
