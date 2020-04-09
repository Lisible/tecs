use std::any::{Any, TypeId};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Ecs {}
impl Ecs {
    pub fn new() -> Ecs {
        Ecs {}
    }

    pub fn new_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }
}

pub struct EntityBuilder<'a> {
    ecs: &'a mut Ecs,
    definition: HashSet<TypeId>,
    components: Vec<Box<dyn Any>>,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(ecs: &'a mut Ecs) -> Self {
        EntityBuilder {
            ecs,
            definition: HashSet::new(),
            components: vec![],
        }
    }

    pub fn with_component(mut self, component: impl Any) -> Self {
        self.definition.insert(component.type_id());
        self.components.push(Box::new(component));
        self
    }

    pub fn build(self) {
        // TODO Store the entity
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Entity {
    identifier: usize,
}

impl Entity {
    pub fn new(identifier: usize) -> Entity {
        Entity { identifier }
    }

    pub fn identifier(&self) -> usize {
        self.identifier
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
    struct Speed {
        pub x: f32,
        pub y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Health {
        pub health: f32,
    }

    #[test]
    pub fn build_entity() {
        let mut ecs = Ecs::new();
        ecs.new_entity().build();
    }
}
