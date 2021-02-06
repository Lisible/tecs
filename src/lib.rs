use std::alloc::GlobalAlloc;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ptr::NonNull;

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
        let _entity_id = self.entity_store.allocate_entity();
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
            .or_insert(Archetype::new::<D>())
    }
}

pub struct Archetype {
    components_metadata: ComponentsMetadata,
    data: NonNull<u8>,
    entity_count: usize,
    capacity: usize,
}

impl Archetype {
    pub fn new<C: ComponentsDefinition>() -> Self {
        Self {
            components_metadata: C::metadata(),
            data: NonNull::dangling(),
            entity_count: 0,
            capacity: 0,
        }
    }

    pub fn store_component(&mut self, component_data: *const u8, type_index: usize) {
        if self.capacity < self.entity_count + 1 {
            self.grow();
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                component_data,
                self.data.as_ptr().offset(self.entity_count as isize),
                1,
            )
        }

        if type_index == 0 {
            self.entity_count += 1;
        }
    }

    fn grow(&mut self) {
        unsafe {
            let _entity_alignment = self.components_metadata.entity_alignment;
            let _entity_size = self.components_metadata.entity_size;
            let entity_layout = self.components_metadata.entity_layout;

            // TODO handle realloc
            let (new_capacity, ptr) = {
                let ptr = std::alloc::System.alloc(entity_layout);
                (1, ptr)
            };

            // TODO handle error
            self.data = NonNull::new_unchecked(ptr);
            self.capacity = new_capacity;
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entity_count
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

pub trait ComponentsDefinition {
    fn component_types() -> Box<[ComponentType]>;
    fn metadata() -> ComponentsMetadata;
    fn store_components(&mut self, archetype: &mut Archetype);
}

impl<A: 'static, B: 'static> ComponentsDefinition for (A, B) {
    fn component_types() -> Box<[ComponentType]> {
        Box::new([TypeId::of::<A>(), TypeId::of::<B>()])
    }

    fn metadata() -> ComponentsMetadata {
        let mut types_metadata = vec![];
        types_metadata.push(TypeMetadata);
        types_metadata.push(TypeMetadata);

        ComponentsMetadata {
            entity_size: std::mem::size_of::<(A, B)>(),
            entity_alignment: std::mem::align_of::<(A, B)>(),
            entity_layout: std::alloc::Layout::new::<(A, B)>(),
            _types_metadata: types_metadata,
        }
    }
    fn store_components(&mut self, archetype: &mut Archetype) {
        archetype.store_component(&self.0 as *const A as *const u8, 0usize);
        archetype.store_component(&self.1 as *const B as *const u8, 1usize);
    }
}

pub struct ComponentsMetadata {
    entity_size: usize,
    entity_alignment: usize,
    entity_layout: std::alloc::Layout,
    _types_metadata: Vec<TypeMetadata>,
}

// TODO figure out what to store in there
pub struct TypeMetadata;

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

    #[test]
    pub fn archetype_new() {
        let archetype = Archetype::new::<(Position, Velocity)>();
        assert_eq!(archetype.components_metadata._types_metadata.len(), 2);
    }

    #[test]
    pub fn archetype_store() {
        let mut archetype = Archetype::new::<(Position, Velocity)>();
        (Position { x: 3f32, y: 5f32 }, Velocity { x: 8f32, y: 6f32 })
            .store_components(&mut archetype);
        assert_eq!(archetype.entity_count(), 1);
    }
}
