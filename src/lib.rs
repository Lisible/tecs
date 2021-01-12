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

    pub fn create_entity<T: EntityDefinition>(&mut self, entity: T) -> EntityId {
        self.component_store.create_entity(entity)
    }

    pub fn entity_count(&self) -> usize {
        0
    }
}

#[derive(Debug)]
struct ComponentStore {
    archetypes: HashMap<ArchetypeDescription, ArchetypeData>,
}

impl ComponentStore {
    pub fn new() -> Self {
        ComponentStore {
            archetypes: HashMap::new(),
        }
    }

    pub fn create_entity<T: EntityDefinition>(&mut self, entity: T) -> EntityId {
        let archetype_description = T::archetype_description();
        let archetype_data = self
            .archetypes
            .entry(archetype_description)
            .or_insert(ArchetypeData);

        archetype_data.insert(entity)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeDescription;

#[derive(Debug)]
pub struct ArchetypeData;
impl ArchetypeData {
    pub fn new() -> Self {
        Self
    }

    pub fn insert<T>(&mut self, entity: T) -> EntityId {
        0
    }
}

pub trait EntityDefinition {
    fn archetype_description() -> ArchetypeDescription;
}

impl<A, B> EntityDefinition for (A, B) {
    fn archetype_description() -> ArchetypeDescription {
        ArchetypeDescription
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
    pub fn ecs_create_entity() {
        let mut ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
        ecs.create_entity((Position { x: 0f32, y: 0f32 }, Velocity { x: 0f32, y: 0f32 }));
        assert_eq!(ecs.entity_count(), 1);
    }
}
