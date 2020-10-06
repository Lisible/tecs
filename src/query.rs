use crate::core::{ComponentIter, ComponentIterMut, Ecs};
use itertools::{multizip, Zip};
use std::marker::PhantomData;

/// Mutable accessor for components
pub struct Mut<T>(PhantomData<T>);
/// Immutable accessor for components
pub struct Imm<T>(PhantomData<T>);

pub trait Queryable<'a> {
    type Iter: Iterator + 'a;

    fn fetch(ecs: *mut Ecs) -> Self::Iter;
}

impl<'a, T: 'static> Queryable<'a> for Mut<T> {
    type Iter = ComponentIterMut<'a, T>;

    fn fetch(ecs: *mut Ecs) -> Self::Iter {
        unsafe { ecs.as_mut().unwrap().component_iter_mut::<T>() }
    }
}

impl<'a, T: 'static> Queryable<'a> for Imm<T> {
    type Iter = ComponentIter<'a, T>;

    fn fetch(ecs: *mut Ecs) -> Self::Iter {
        unsafe { ecs.as_mut().unwrap().component_iter::<T>() }
    }
}

macro_rules! tuple_queryable_impl {
    ($($ty:ident,)*) => {
        impl<'a, $($ty: Queryable<'a>,)*> Queryable<'a> for ($($ty,)*) {
            type Iter = Zip<($($ty::Iter,)*)>;

            fn fetch(ecs: *mut Ecs) -> Self::Iter {
                multizip(($($ty::fetch(ecs),)*))
            }
        }
    };
}

tuple_queryable_impl!(A,);
tuple_queryable_impl!(A, B,);
tuple_queryable_impl!(A, B, C,);
tuple_queryable_impl!(A, B, C, D,);
tuple_queryable_impl!(A, B, C, D, E,);
tuple_queryable_impl!(A, B, C, D, E, F,);
tuple_queryable_impl!(A, B, C, D, E, F, G,);
tuple_queryable_impl!(A, B, C, D, E, F, G, H,);

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
    pub fn ecs_query() {
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

        assert_eq!(<(Mut<Position>, Imm<Speed>)>::fetch(&mut ecs).count(), 3);
        assert_eq!(<(Mut<Position>, Imm<Health>)>::fetch(&mut ecs).count(), 2);
        assert_eq!(
            <(Mut<Position>, Imm<Health>, Imm<Burnable>)>::fetch(&mut ecs).count(),
            2
        );

        assert_eq!(
            <(Mut<Position>, Imm<Speed>)>::fetch(&mut ecs).next(),
            Some((&mut Position { x: 0.5, y: 2.3 }, &Speed { x: 1.0, y: 4.0 }))
        );
    }
}
