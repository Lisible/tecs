use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::slice::{Iter, IterMut};

type EntityId = usize;

/// Contains the entire Ecs state
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
    /// use tecs::core::Ecs;
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
    /// use tecs::core::Ecs;
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
    /// use tecs::core::Ecs;
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
    /// use tecs::core::Ecs;
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

    /// Returns an iterator for the given component
    ///
    /// # Examples
    ///
    /// ```
    /// use tecs::core::Ecs;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32
    /// }
    ///
    /// let mut ecs = Ecs::new();
    /// ecs.new_entity()
    ///     .with_component(Position { x: 1.0, y: 2.0 })
    ///     .build();
    ///
    /// ecs.new_entity()
    ///     .with_component(Position { x: 3.0, y: 4.0 })
    ///     .build();
    ///
    /// ecs.new_entity()
    ///     .with_component(Position { x: 5.0, y: 6.0 })
    ///     .build();
    ///
    /// let component_iterator = ecs.component_iter::<Position>();
    /// assert_eq!(component_iterator.count(), 3);
    ///
    /// ```
    pub fn component_iter<T: 'static>(&mut self) -> ComponentIter<'_, T> {
        ComponentIter::new(self)
    }

    /// Returns a mutable iterator for the given component
    ///
    /// # Examples
    ///
    /// ```
    /// use tecs::core::Ecs;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Position {
    ///     x: f32,
    ///     y: f32
    /// }
    ///
    /// let mut ecs = Ecs::new();
    /// ecs.new_entity()
    ///     .with_component(Position { x: 1.0, y: 2.0 })
    ///     .build();
    ///
    /// ecs.new_entity()
    ///     .with_component(Position { x: 3.0, y: 4.0 })
    ///     .build();
    ///
    /// ecs.new_entity()
    ///     .with_component(Position { x: 5.0, y: 6.0 })
    ///     .build();
    ///
    /// let component_iterator = ecs.component_iter_mut::<Position>();
    /// assert_eq!(component_iterator.count(), 3);
    ///
    /// ```
    pub fn component_iter_mut<T: 'static>(&mut self) -> ComponentIterMut<'_, T> {
        ComponentIterMut::new(self)
    }

    fn fetch_next_entity_id(&mut self) -> EntityId {
        if let Some(id) = self.entity_free_list.pop() {
            id
        } else {
            let id = self.next_entity_id;
            self.resize_component_stores();
            self.next_entity_id += 1;
            id
        }
    }

    fn resize_component_stores(&mut self) {
        for storage in self.components.values_mut() {
            storage.resize_with(self.next_entity_id + 1, || None);
        }
    }
}

pub struct ComponentIter<'a, T> {
    iterator: Iter<'a, Option<Box<dyn Any>>>,
    phantom: PhantomData<T>,
}

pub struct ComponentIterMut<'a, T> {
    iterator: IterMut<'a, Option<Box<dyn Any>>>,
    phantom: PhantomData<T>,
}

impl<'a, T: 'static> ComponentIter<'a, T> {
    pub fn new(ecs: &'a mut Ecs) -> ComponentIter<'a, T> {
        ComponentIter {
            iterator: ecs.components.get(&TypeId::of::<T>()).unwrap().iter(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'static> ComponentIterMut<'a, T> {
    pub fn new(ecs: &'a mut Ecs) -> ComponentIterMut<'a, T> {
        ComponentIterMut {
            iterator: ecs
                .components
                .get_mut(&TypeId::of::<T>())
                .unwrap()
                .iter_mut(),
            phantom: PhantomData,
        }
    }
}

impl<'a, T: 'static> Iterator for ComponentIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.find_map(|x| x.as_ref())?.downcast_ref()
    }
}

impl<'a, T: 'static> Iterator for ComponentIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.find_map(|x| x.as_mut())?.downcast_mut()
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
    pub fn component_iter() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 2.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 0.0, y: 2.0 })
            .with_component(Health { health: 15.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.1, y: 2.5 })
            .with_component(Speed { x: 0.5, y: 1.3 })
            .with_component(Health { health: 12.0 })
            .build();

        assert_eq!(
            *ecs.component_iter::<Position>().nth(0).unwrap(),
            Position { x: 0.5, y: 2.3 }
        );
        assert_eq!(
            *ecs.component_iter::<Position>().nth(1).unwrap(),
            Position { x: 0.0, y: 2.0 }
        );
        assert_eq!(
            *ecs.component_iter::<Position>().nth(2).unwrap(),
            Position { x: 1.1, y: 2.5 }
        );
    }

    #[test]
    pub fn component_iter_mut() {
        let mut ecs = Ecs::new();
        ecs.new_entity()
            .with_component(Position { x: 0.5, y: 2.3 })
            .with_component(Speed { x: 1.0, y: 2.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 0.0, y: 2.0 })
            .with_component(Health { health: 15.0 })
            .build();

        ecs.new_entity()
            .with_component(Position { x: 1.1, y: 2.5 })
            .with_component(Speed { x: 0.5, y: 1.3 })
            .with_component(Health { health: 12.0 })
            .build();

        for position in ecs.component_iter_mut::<Position>() {
            position.y = 0.0;
        }

        assert_eq!(
            *ecs.component::<Position>(0).unwrap(),
            Position { x: 0.5, y: 0.0 }
        );
        assert_eq!(
            *ecs.component::<Position>(1).unwrap(),
            Position { x: 0.0, y: 0.0 }
        );
        assert_eq!(
            *ecs.component::<Position>(2).unwrap(),
            Position { x: 1.1, y: 0.0 }
        );
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