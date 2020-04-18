use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

type EntityId = usize;

/// Contains the entire Ecs state
#[derive(Debug)]
pub struct Ecs {
    next_entity_id: EntityId,
    entity_free_list: Vec<EntityId>,
    components: HashMap<TypeId, Vec<Option<Box<dyn Any>>>>,
}

impl Ecs {
    /// Create an empty `Ecs`.
    pub fn new() -> Ecs {
        Ecs {
            next_entity_id: 0,
            entity_free_list: vec![],
            components: HashMap::new(),
        }
    }

    /// Create a new entity in the Ecs.
    /// This function will return an `EntityBuilder`, the entity will be stored
    /// as soon as `EntityBuilder::build` is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use tecs::*;
    ///
    /// struct Position {
    ///     x: f32,
    ///     y: f32
    /// }
    ///
    /// let mut ecs = Ecs::new();
    /// let entity_id = ecs.new_entity()
    ///     .with_component(Position { x: 1.0, y: 2.0 })
    ///     .build();
    ///
    /// assert!(ecs.component::<Position>(0).is_some())
    /// ```
    pub fn new_entity(&mut self) -> EntityBuilder {
        EntityBuilder::new(self)
    }

    /// Remove an entity from the Ecs.
    ///
    /// This will set all the entity components to None and add the entity id
    /// to the entity id free list for reuse of the id.
    ///
    /// # Examples
    ///
    /// ```
    /// use tecs::*;
    ///
    /// struct Position {
    ///     x: f32,
    ///     y: f32
    /// }
    ///
    /// let mut ecs = Ecs::new();
    /// let first_entity_id = ecs.new_entity()
    ///     .with_component(Position { x: 1.0, y: 2.0 })
    ///     .build();
    /// let second_entity_id = ecs.new_entity()
    ///     .with_component(Position { x: 3.0, y: 4.0 })
    ///     .build();
    ///
    /// assert!(ecs.component::<Position>(first_entity_id).is_some());
    /// assert!(ecs.component::<Position>(second_entity_id).is_some());
    ///
    /// ecs.remove_entity(first_entity_id);
    ///
    /// assert!(ecs.component::<Position>(first_entity_id).is_none());
    /// assert!(ecs.component::<Position>(second_entity_id).is_some());
    ///
    /// let new_entity_id = ecs.new_entity()
    ///     .with_component(Position { x: 5.0, y: 6.0 })
    ///     .build();
    ///
    /// assert_eq!(new_entity_id, first_entity_id);
    /// ```
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        for component in self.components.values_mut() {
            component[entity_id] = None;
        }

        self.entity_free_list.push(entity_id);
    }

    /// Returns a reference to the component of an entity
    ///
    /// # Examples
    ///
    /// ```
    /// use tecs::*;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32
    /// }
    ///
    /// let mut ecs = Ecs::new();
    /// let entity = ecs.new_entity()
    ///     .with_component(Position { x: 3.0, y: 4.5 })
    ///     .build();
    ///
    /// assert_eq!(*ecs.component::<Position>(entity).unwrap(), Position { x: 3.0, y: 4.5 });
    /// ```
    pub fn component<T: 'static>(&self, entity_id: EntityId) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .get(entity_id)?
            .as_ref()?
            .downcast_ref()
    }
    /// Returns a mutable reference to the component of an entity
    ///
    /// # Examples
    ///
    /// ```
    /// use tecs::*;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32
    /// }
    ///
    /// let mut ecs = Ecs::new();
    /// let entity = ecs.new_entity()
    ///     .with_component(Position { x: 3.0, y: 4.5 })
    ///     .build();
    ///
    /// assert_eq!(*ecs.component::<Position>(entity).unwrap(), Position { x: 3.0, y: 4.5 });
    ///
    /// ecs.component_mut::<Position>(entity).unwrap().x = 200.0;
    /// assert_eq!(*ecs.component::<Position>(entity).unwrap(), Position { x: 200.0, y: 4.5 });
    /// ```
    pub fn component_mut<T: 'static>(&mut self, entity_id: EntityId) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())?
            .get_mut(entity_id)?
            .as_mut()?
            .downcast_mut()
    }

    fn fetch_next_entity_id(&mut self) -> EntityId {
        if self.entity_free_list.is_empty() {
            let id = self.next_entity_id;
            self.resize_component_stores();
            self.next_entity_id += 1;
            id
        } else {
            self.entity_free_list
                .pop()
                .expect("No entity id in freelist")
        }
    }

    fn resize_component_stores(&mut self) {
        for storage in self.components.values_mut() {
            storage.resize_with(self.next_entity_id + 1, || None);
        }
    }
}

/// Builds an entity with a given set of components
pub struct EntityBuilder<'a> {
    ecs: &'a mut Ecs,
    components: Vec<Box<dyn Any>>,
}

impl<'a> EntityBuilder<'a> {
    /// Create a new `EntityBuilder` for the given `Ecs`.
    pub fn new(ecs: &'a mut Ecs) -> Self {
        EntityBuilder {
            ecs,
            components: vec![],
        }
    }

    /// Add a component to the entity that is being created
    pub fn with_component(mut self, component: impl Any) -> Self {
        self.components.push(Box::new(component));
        self
    }

    /// Build the entity with its component.
    ///
    /// This methods effectively stores the components into the components
    /// storage. If no storage is available for a given component, it is
    /// created.
    ///
    /// Returns the id of the newly created entity.
    pub fn build(self) -> EntityId {
        let id = self.ecs.fetch_next_entity_id();
        for component in self.components {
            let type_id = (*component).type_id();
            if let Some(storage) = self.ecs.components.get_mut(&type_id) {
                storage[id] = Some(component);
            } else {
                let mut storage = vec![];
                storage.resize_with(id + 1, || None);
                storage[id] = Some(component);
                self.ecs.components.insert((&type_id).clone(), storage);
            }
        }

        id
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut count = 0;
        for i in 0..self.ecs.next_entity_id {
            if self.component_type_ids.iter().all(|type_id| {
                self.ecs.components.get(type_id).is_some()
                    && self
                        .ecs
                        .components
                        .get(type_id)
                        .expect("Unknown component type")
                        .get(i)
                        .expect(format!("No component at index {}", self.index).as_str())
                        .is_some()
            }) {
                count += 1;
            }
        }

        (count, Some(count))
    }
}

impl<'a, Q: Query<'a>> ExactSizeIterator for QueryIter<'a, Q> where Self: Sized {}

pub trait Query<'a> {
    type Iter: From<QueryResult<'a>>;

    fn iter(ecs: &'a Ecs) -> QueryIter<'a, Self>
    where
        Self: Sized;
}

pub struct QueryResult<'a>(Vec<&'a Box<dyn Any>>);

macro_rules! tuple_query_impl {
    ($($idx:expr => $type:ident,)+) => {
            impl<'a, $($type: 'static,)+> Query<'a> for ($($type,)+) {
                type Iter = ($(&'a $type,)+);

                fn iter(ecs: &'a Ecs) -> QueryIter<'a, Self> {
                    QueryIter {
                        index: 0,
                        component_type_ids: vec![$(TypeId::of::<$type>(),)+],
                        ecs,
                        query: PhantomData,
                    }
                }
            }

            impl<'a, $($type: 'static,)+> From<QueryResult<'a>> for ($(&'a $type,)+) {
                fn from(result: QueryResult<'a>) -> Self {
                    ($(result.0[$idx].downcast_ref().unwrap(),)*)
                }
            }
    };
}

tuple_query_impl!(0 => A,);
tuple_query_impl!(0 => A, 1 => B,);
tuple_query_impl!(0 => A, 1 => B, 2 => C,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T, 20 => U,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T, 20 => U, 21 => V,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T, 20 => U, 21 => V, 22 => W,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T, 20 => U, 21 => V, 22 => W, 23 => X,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T, 20 => U, 21 => V, 22 => W, 23 => X, 24 => Y,);
tuple_query_impl!(0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F, 6 => G, 7 => H, 8 => I, 9 => J, 10 => K, 11 => L, 12 => M, 13 => N, 14 => O, 15 => P, 16 => Q, 17 => R, 18 => S, 19 => T, 20 => U, 21 => V, 22 => W, 23 => X, 24 => Y, 25 => Z,);

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

    #[derive(Debug, PartialEq)]
    struct Burnable;

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

        assert_eq!(
            <(Position, Speed)>::iter(&ecs).nth(0),
            Some((&Position { x: 0.5, y: 2.3 }, &Speed { x: 1.0, y: 4.0 }))
        );

        assert_eq!(
            <(Position, Speed)>::iter(&ecs).nth(1),
            Some((&Position { x: 1.0, y: 2.3 }, &Speed { x: 12.5, y: 80.0 }))
        );
    }

    #[test]
    pub fn query3() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.0, y: 2.3 })
            .with_component(Health { health: 100.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.0, y: 2.3 })
            .with_component(Speed { x: 12.5, y: 80.0 })
            .with_component(Health { health: 95.0 })
            .build();

        assert_eq!(
            <(Position, Speed, Health)>::iter(&ecs).nth(0),
            Some((
                &Position { x: 1.0, y: 2.3 },
                &Speed { x: 12.5, y: 80.0 },
                &Health { health: 95.0 }
            ))
        );

        assert_eq!(<(Position, Speed, Health)>::iter(&ecs).nth(1), None);
        assert_eq!(<(Position, Speed, Health)>::iter(&ecs).len(), 1);
        assert_eq!(<(Position, Health)>::iter(&ecs).len(), 2);
    }

    #[test]
    pub fn query4() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.0, y: 2.3 })
            .with_component(Speed { x: 12.0, y: 42.0 })
            .with_component(Health { health: 100.0 })
            .with_component(Burnable)
            .build();

        ecs.new_entity()
            .with_component(Position { x: 18.2, y: 4.5 })
            .with_component(Speed { x: 122.0, y: 12.0 })
            .with_component(Health { health: 95.0 })
            .with_component(Burnable)
            .build();

        ecs.new_entity().with_component(Burnable).build();

        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 18.54, y: 4.5 })
            .with_component(Speed { x: 122.0, y: 12.0 })
            .with_component(Health { health: 95.0 })
            .with_component(Burnable)
            .build();

        assert_eq!(<(Position,)>::iter(&ecs).len(), 5);
    }

    #[test]
    pub fn ecs_remove_entity() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 4.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.0, y: 2.3 })
            .with_component(Speed { x: 12.0, y: 42.0 })
            .with_component(Health { health: 100.0 })
            .with_component(Burnable)
            .build();

        ecs.new_entity()
            .with_component(Position { x: 18.2, y: 4.5 })
            .with_component(Speed { x: 122.0, y: 12.0 })
            .with_component(Health { health: 95.0 })
            .with_component(Burnable)
            .build();

        ecs.remove_entity(1);
        ecs.remove_entity(0);

        assert_eq!(ecs.new_entity().build(), 0);
        assert_eq!(
            ecs.new_entity()
                .with_component(Position { x: 15.0, y: 23.0 })
                .build(),
            1
        );
        assert_eq!(ecs.new_entity().build(), 3);

        for &i in [0usize, 3].iter() {
            assert!(ecs.component::<Position>(i).is_none());
            assert!(ecs.component::<Speed>(i).is_none());
            assert!(ecs.component::<Health>(i).is_none());
            assert!(ecs.component::<Burnable>(i).is_none());
        }

        assert!(ecs.component::<Position>(1).is_some());
    }
}
