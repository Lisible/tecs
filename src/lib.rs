use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

type EntityId = usize;

#[derive(Debug)]
pub struct Ecs {
    next_entity_id: EntityId,
    components: HashMap<TypeId, Vec<Option<Box<dyn Any>>>>,
}
impl Ecs {
    pub fn new() -> Ecs {
        Ecs {
            next_entity_id: 0,
            components: HashMap::new(),
        }
    }

    pub fn new_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }
    pub fn component<T: 'static>(&self, index: usize) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .get(index)?
            .as_ref()?
            .downcast_ref()
    }
    pub fn component_mut<T: 'static>(&mut self, index: usize) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(index)?
            .as_mut()?
            .downcast_mut()
    }

    fn resize_component_stores(&mut self) {
        for storage in self.components.values_mut() {
            storage.resize_with(self.next_entity_id + 1, || None);
        }
    }
}

pub struct EntityBuilder<'a> {
    ecs: &'a mut Ecs,
    components: Vec<Box<dyn Any>>,
}

impl<'a> EntityBuilder<'a> {
    pub fn new(ecs: &'a mut Ecs) -> Self {
        EntityBuilder {
            ecs,
            components: vec![],
        }
    }

    pub fn with_component(mut self, component: impl Any) -> Self {
        self.components.push(Box::new(component));
        self
    }

    pub fn build(self) -> EntityId {
        self.ecs.resize_component_stores();
        let id = self.ecs.next_entity_id;
        for component in self.components {
            if let Some(storage) = self.ecs.components.get_mut(&(*component).type_id()) {
                storage[id] = Some(component);
            } else {
                self.ecs
                    .components
                    .insert(((*component).type_id()).clone(), vec![]);
                let storage = self
                    .ecs
                    .components
                    .get_mut(&(*component).type_id())
                    .expect("component storage");
                storage.push(Some(component));
            }
        }

        self.ecs.next_entity_id += 1;
        id
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

pub struct QueryIter<'a, Q> {
    index: usize,
    component_type_ids: Vec<TypeId>,
    ecs: &'a Ecs,
    query: PhantomData<Q>,
}

impl<'a, Q: Query<'a>> Iterator for QueryIter<'a, Q>
where
    Self: Sized,
{
    type Item = Q::Iter;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.ecs.next_entity_id {
            return None;
        }

        while self.component_type_ids.iter().any(|type_id| {
            self.ecs.components.get(type_id).is_none()
                || self
                    .ecs
                    .components
                    .get(type_id)
                    .expect("Unknown component type")
                    .get(self.index)
                    .expect(format!("No component at index {}", self.index).as_str())
                    .is_none()
        }) {
            self.index += 1;
            if self.index >= self.ecs.next_entity_id {
                return None;
            }
        }

        let mut result = vec![];
        for type_id in &self.component_type_ids {
            result.push(
                self.ecs
                    .components
                    .get(type_id)?
                    .get(self.index)?
                    .as_ref()?,
            )
        }

        self.index += 1;
        Some(Q::Iter::from(QueryResult(result)))
    }
}

pub trait Query<'a> {
    type Iter: From<QueryResult<'a>>;

    fn iter(ecs: &'a Ecs) -> QueryIter<'a, Self>
    where
        Self: Sized;
}

impl<'a, A: 'static, B: 'static> Query<'a> for (A, B) {
    type Iter = (&'a A, &'a B);

    fn iter(ecs: &'a Ecs) -> QueryIter<'a, Self> {
        QueryIter {
            index: 0,
            component_type_ids: vec![TypeId::of::<A>(), TypeId::of::<B>()],
            ecs,
            query: PhantomData,
        }
    }
}

pub struct QueryResult<'a>(Vec<&'a Box<dyn Any>>);

impl<'a, A: 'static, B: 'static> From<QueryResult<'a>> for (&'a A, &'a B) {
    fn from(result: QueryResult<'a>) -> Self {
        (
            result.0[0].downcast_ref().unwrap(),
            result.0[1].downcast_ref().unwrap(),
        )
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
    pub fn ecs_build_entity() {
        let mut ecs = Ecs::new();
        ecs.new_entity().build();
    }

    #[test]
    pub fn ecs_component() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        assert_eq!(
            *ecs.component::<Position>(0).unwrap(),
            Position { x: 0.5, y: 2.3 }
        );
    }

    #[test]
    pub fn ecs_component_mut() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        assert_eq!(
            *ecs.component::<Position>(0).unwrap(),
            Position { x: 0.5, y: 2.3 }
        );

        ecs.component_mut::<Position>(0).unwrap().x = 100.0;
        ecs.component_mut::<Position>(0).unwrap().y = 976.5;

        assert_eq!(
            *ecs.component::<Position>(0).unwrap(),
            Position { x: 100.0, y: 976.5 }
        );
    }

    #[test]
    pub fn query() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.0, y: 2.3 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.0, y: 2.3 })
            .with_component(Speed { x: 12.5, y: 80.0 })
            .build();

        for (position, speed) in <(Position, Speed)>::iter(&mut ecs) {
            println!("{:?}", position);
            println!("{:?}", speed);
        }

        assert_eq!(
            <(Position, Speed)>::iter(&ecs).nth(0),
            Some((&Position { x: 0.5, y: 2.3 }, &Speed { x: 1.0, y: 4.0 }))
        );

        assert_eq!(
            <(Position, Speed)>::iter(&ecs).nth(1),
            Some((&Position { x: 1.0, y: 2.3 }, &Speed { x: 12.5, y: 80.0 }))
        );
    }
}
