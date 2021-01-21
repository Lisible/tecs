use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ptr::NonNull;

pub type EntityId = usize;
pub type Component = Box<dyn Any>;
pub type ComponentType = TypeId;

pub struct Ecs {
    archetypes: HashMap<Box<[ComponentType]>, Archetype>,
}

impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            archetypes: HashMap::new(),
        }
    }

    pub fn create_entity<D: ComponentsDefinition>(&mut self, components_definition: D) {
        let archetype = self.get_or_insert_archetype::<D>();
        archetype.create_entity(components_definition);
    }

    pub fn entity_count(&self) -> usize {
        self.archetypes.values().map(|a| a.entity_count()).sum()
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

pub struct Archetype {
    archetype_data: NonNull<u8>,
    entity_count: usize,
}
impl Archetype {
    pub fn new() -> Self {
        Self {
            archetype_data: NonNull::dangling(),
            entity_count: 0,
        }
    }

    pub fn create_entity<D: ComponentsDefinition>(&mut self, components_definition: D) {
        self.entity_count += 1;
    }

    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
}

impl Default for Archetype {
    fn default() -> Self {
        Self::new()
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
    pub fn ecs_create_entity() {
        let mut ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0);
        ecs.create_entity((Position { x: 0f32, y: 0f32 }, Velocity { x: 0f32, y: 0f32 }));
        assert_eq!(ecs.entity_count(), 1);
        assert_eq!(
            ecs.archetype::<(Position, Velocity)>()
                .iter_entities()
                .next(),
            Some(&(Position { x: 0f32, y: 0f32 }, Velocity { x: 0f32, y: 0f32 }))
        );
    }
}
