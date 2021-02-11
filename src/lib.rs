use std::alloc::Layout;
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
    size: usize,
    stored_entities: Vec<EntityId>,
    entity_count: usize,
    types_offset: Vec<usize>,
    capacity: usize,
}

impl Archetype {
    pub fn new<C: ComponentsDefinition>() -> Self {
        Self {
            components_metadata: C::metadata(),
            data: NonNull::dangling(),
            size: 0,
            stored_entities: vec![],
            entity_count: 0,
            types_offset: vec![],
            capacity: 0,
        }
    }

    pub fn allocate_storage_for_entity(&mut self, entity_id: EntityId) -> usize {
        if self.size == self.capacity {
            if self.capacity == 0 {
                self.grow(1);
            } else {
                self.grow(self.capacity * 2);
            }
        }

        self.stored_entities.push(entity_id);
        self.entity_count += 1;
        self.stored_entities.len() - 1
    }

    // This code is heavily inspired from hecs archetype grow method
    // https://github.com/Ralith/hecs/blob/master/src/archetype.rs
    fn grow(&mut self, new_capacity: usize) {
        let new_entity_count = self.entity_count + new_capacity;

        // First we resize the stored_entity vec
        self.stored_entities.resize_with(new_capacity, || 0);

        // Then we compute the required size to store correctly aligned components
        let mut types_offset = vec![0; self.components_metadata.types_metadata.len()];
        let mut new_size = 0;
        for (i, type_metadata) in self.components_metadata.types_metadata.iter().enumerate() {
            new_size = align(new_size, type_metadata.alignment);
            types_offset[i] = new_size;
            new_size += type_metadata.size * new_entity_count;
        }

        // Then we allocate that space
        let mut new_data: NonNull<u8> = NonNull::dangling();
        unsafe {
            if new_capacity > 0 {
                new_data = NonNull::new(std::alloc::alloc(
                    Layout::from_size_align(
                        new_size,
                        self.components_metadata
                            .types_metadata
                            .first()
                            .map_or(1, |t| t.alignment),
                    )
                    .unwrap(),
                ))
                .unwrap();
            }
        }

        // TODO free reallocated data

        self.size = new_size;
        self.data = new_data;
        self.types_offset = types_offset;
    }

    pub unsafe fn store_component(
        &mut self,
        component_data: *const u8,
        type_index: usize,
        data_index: usize,
        data_size: usize,
    ) {
        let destination_ptr = NonNull::new_unchecked(
            self.data
                .as_ptr()
                .add(self.types_offset[type_index] + data_size * data_index)
                .cast::<u8>(),
        );
        std::ptr::copy_nonoverlapping(component_data, destination_ptr.as_ptr(), data_size);
    }

    pub fn entity_count(&self) -> usize {
        self.entity_count
    }
}

fn align(value: usize, alignment: usize) -> usize {
    (value + alignment - 1) & (!alignment - 1)
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
    fn store_components(&mut self, archetype: &mut Archetype, index: usize);
}

impl<A: 'static, B: 'static> ComponentsDefinition for (A, B) {
    fn component_types() -> Box<[ComponentType]> {
        Box::new([TypeId::of::<A>(), TypeId::of::<B>()])
    }

    fn metadata() -> ComponentsMetadata {
        let mut types_metadata = vec![];
        types_metadata.push(TypeMetadata {
            alignment: std::mem::align_of::<A>(),
            size: std::mem::size_of::<A>(),
        });
        types_metadata.push(TypeMetadata {
            alignment: std::mem::align_of::<B>(),
            size: std::mem::size_of::<B>(),
        });

        ComponentsMetadata {
            entity_size: std::mem::size_of::<(A, B)>(),
            entity_alignment: std::mem::align_of::<(A, B)>(),
            entity_layout: std::alloc::Layout::new::<(A, B)>(),
            types_metadata: types_metadata,
        }
    }
    fn store_components(&mut self, archetype: &mut Archetype, index: usize) {
        unsafe {
            archetype.store_component(
                &self.0 as *const A as *const u8,
                0usize,
                index,
                std::mem::size_of::<A>(),
            );
            archetype.store_component(
                &self.1 as *const B as *const u8,
                1usize,
                index,
                std::mem::size_of::<B>(),
            );
        }
    }
}

pub struct ComponentsMetadata {
    entity_size: usize,
    entity_alignment: usize,
    entity_layout: std::alloc::Layout,
    types_metadata: Vec<TypeMetadata>,
}

pub struct TypeMetadata {
    alignment: usize,
    size: usize,
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
        assert_eq!(archetype.components_metadata.types_metadata.len(), 2);
    }

    #[test]
    pub fn archetype_store() {
        let mut archetype = Archetype::new::<(Position, Velocity)>();
        let index = archetype.allocate_storage_for_entity(1);
        (Position { x: 3f32, y: 5f32 }, Velocity { x: 8f32, y: 6f32 })
            .store_components(&mut archetype, index);
        assert_eq!(archetype.entity_count(), 1);
    }
}
