use crate::core::Ecs;
use crate::query::Queryable;
use std::marker::PhantomData;

/// A System for a specific query
pub struct System<'a, Q: Queryable<'a>> {
    query: PhantomData<Q>,
    function: Box<dyn FnMut(<<Q as Queryable<'a>>::Iter as Iterator>::Item)>,
}

impl<'a, Q: Queryable<'a>> System<'a, Q> {
    pub fn new(
        f: impl Fn(<<Q as Queryable<'a>>::Iter as Iterator>::Item) + 'static,
    ) -> System<'a, Q> {
        System {
            query: PhantomData,
            function: Box::new(f),
        }
    }
}

impl<'a, Q: Queryable<'a>> Runnable for System<'a, Q> {
    fn run(&mut self, ecs: &mut Ecs) {
        for p in Q::fetch(ecs) {
            (self.function)(p);
        }
    }
}

pub trait Runnable {
    fn run(&mut self, ecs: &mut Ecs);
}

/// Holds a system list and runs them on an Ecs
pub struct SystemSchedule {
    systems: Vec<Box<dyn Runnable>>,
}

impl SystemSchedule {
    /// Runs the schedule
    pub fn run(&mut self, ecs: &mut Ecs) {
        for system in &mut self.systems {
            system.run(ecs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{Imm, Mut};

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
    pub fn ecs_system() {
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

        let mut heal_system = System::<(Mut<Health>,)>::new(|(health,)| {
            health.health = 100.0;
        });

        heal_system.run(&mut ecs);

        for (health,) in <(Imm<Health>,)>::fetch(&mut ecs) {
            assert_eq!(health.health, 100.0);
        }
    }

    #[test]
    pub fn ecs_system_schedule() {
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

        let heal_system = System::<(Mut<Health>,)> {
            query: PhantomData,
            function: Box::new(|(health,)| {
                health.health = 100.0;
            }),
        };

        let teleport_to_origin = System::<(Mut<Position>,)> {
            query: PhantomData,
            function: Box::new(|(position,)| {
                position.x = 0.0;
                position.y = 0.0;
            }),
        };

        let mut system_schedule = SystemSchedule {
            systems: vec![Box::new(heal_system), Box::new(teleport_to_origin)],
        };

        system_schedule.run(&mut ecs);

        for (position, health) in <(Imm<Position>, Imm<Health>)>::fetch(&mut ecs) {
            assert_eq!(position.x, 0.0);
            assert_eq!(position.y, 0.0);
            assert_eq!(health.health, 100.0);
        }
    }
}
