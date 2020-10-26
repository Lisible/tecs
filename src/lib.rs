use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

pub type EntityId = usize;

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

    pub fn create_entity<Components: 'static + IntoComponentVec>(
        &mut self,
        components: Components,
    ) -> EntityId {
        self.component_store
            .allocate_entity_with_components(components)
    }

    pub fn get<Component: 'static + Debug>(&self, entity_id: EntityId) -> Option<&Component> {
        self.component_store.get_component::<Component>(entity_id)
    }
}

#[derive(Debug)]
struct ComponentStore {
    components_vecs: HashMap<TypeId, Vec<Option<Box<dyn Any>>>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        ComponentStore {
            components_vecs: HashMap::new(),
        }
    }

    pub fn allocate_entity_with_components<Components: 'static + IntoComponentVec>(
        &mut self,
        components: Components,
    ) -> EntityId {
        let components = components.into_component_vec();
        for component in components {
            let type_id = (*component).type_id();
            self.components_vecs
                .entry(type_id)
                .or_insert(vec![])
                .push(Some(component));
        }
        self.components_vecs.iter().next().unwrap().1.len() - 1
    }

    pub fn get_component<Component: 'static + Debug>(
        &self,
        entity_id: EntityId,
    ) -> Option<&Component> {
        self.components_vecs
            .get(&TypeId::of::<Component>())?
            .get(entity_id)?
            .as_ref()?
            .downcast_ref()
    }
}

pub trait IntoComponentVec {
    fn into_component_vec(self) -> Vec<Box<dyn Any>>;
}

impl<A: Any, B: Any> IntoComponentVec for (A, B) {
    fn into_component_vec(self) -> Vec<Box<dyn Any>> {
        vec![Box::new(self.0), Box::new(self.1)]
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
        let entity_id =
            ecs.create_entity((Position { x: 5f32, y: 2f32 }, Velocity { x: 1f32, y: 0f32 }));

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
}
