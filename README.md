# tecs
TeaECS is a simple Rust ECS. I'm building this project for learning purposes. 

This project doesn't have the ambition to be as good or better than popular ECS libraries 
such as [Legion](https://github.com/TomGillen/legion) or [Specs](https://github.com/amethyst/specs). It is however heavily inspired by them.

The entities' data are stored in unique Vecs (one for each component type).

tecs doesn't provide parallel processing features.

* [Using tecs](#using-tecs)
* [Contributing](#contributing)

## Using tecs

### Creating entities

```rust
let mut ecs = Ecs::new();
let entity_id = ecs.new_entity()
    .with_component(Position { x: 0.5, y: 0.3 })
    .with_component(Speed { x: 1.0, y: 2.0 })
    .build();
```

### Removing entities

```rust
ecs.remove_entity(1);
```

### Querying the Ecs

```rust
let mut ecs = Ecs::new();
ecs.new_entity()
    .with_component(Position { x: 0.5, y: 0.3 })
    .with_component(Speed { x: 1.0, y: 2.0 })
    .build();
ecs.new_entity()
    .with_component(Position { x: 1.2, y: 2.2 })
    .with_component(Speed { x: 0.5, y: 0.1 })
    .build();

for (position, speed) in <(Position, Speed)>::iter(&mut ecs) {
    ...
}
```

## Contributing
Feel free to create issues and pull requests to the project.
