![bevy_lit demo](https://github.com/malbernaz/bevy_lit/raw/main/static/demo.png)

# `bevy_lit`

A simple 2D lighting library **designed for Bevy**.

## Features

- Multiple light sources including `PointLight2d`, `SpotLight2d`, `TextureLight2d`, and `DirectionalLight2d`
- Includes primitives `CustomLight2dPlugin` and `Light2dMaterial` for defining custom light sources
- Light occlusion through `LightOccluder2d` that can be used along side any `Mesh2d`
- Per camera fine grain control over lighting parameters such as shadow softness and more
- Terraria-like light penetration effect
- Web support for WebGPU

## Getting started

### Installation

Install it using the CLI:

```sh
cargo add bevy_lit
```

Or add `bevy_lit` to your `Cargo.lock`:

```toml
[dependencies]
bevy_lit = "*"
```

### Usage

Below is a basic example demonstrating how to set up and use `bevy_lit` in your project:

```rust
use bevy::prelude::*;
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        Camera2d,
        Lighting2dSettings::default(),
    ));

    commands.spawn(PointLight2d {
        color: Color::WHITE,
        intensity: 3.0,
        outer_radius: 200.0,
        falloff: 2.0,
        ..default(),
    });

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        LightOccluder2d::default(),
        Transform::from_xyz(0.0, 200.0, 0.0)
    ));
}
```

### Directional light system

`DirectionalLight2d` adds a sun-like light source that projects shadows from each
`LightOccluder2d` based on the occluder height.

- Add one `DirectionalLight2d` entity to your scene.
- `direction` is where light comes from (shadows go the opposite direction).
- `strength` controls projected shadow length (`lower = longer`, `higher = shorter`).
- `tile_size` defines world units used in the height-to-shadow conversion.

Shadow projection formula:

```text
shadow_length = occluder_height * tile_size / strength
```

Use `RoofMask2d` for indoor/covered zones where directional shadows should be suppressed.

```rust,no_run
use bevy::prelude::*;
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Lighting2dSettings::default(),
        AmbientLight2d {
            intensity: 0.2,
            ..default()
        },
    ));

    commands.spawn(DirectionalLight2d {
        color: Color::srgb(1.0, 0.95, 0.82),
        strength: 0.6,
        direction: Vec2::new(0.7, 1.0).normalize(),
        tile_size: 64.0,
    });

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.2))),
        LightOccluder2d {
            height: 16.0,
            ..default()
        },
    ));

    commands.spawn((
        RoofMask2d::new(Vec2::new(220.0, 140.0)),
        Transform::from_xyz(260.0, 0.0, 0.0),
    ));
}
```

See `examples/directional_light.rs` for a complete directional-light scene.

## Compatibility

| `bevy`   | `bevy_lit`  |
| -------- | ----------- |
| `0.18`   | `0.10`       |
| `0.17.3` | `0.9`       |
| `0.17`   | `0.8`       |
| `0.16`   | `0.7`       |
| `0.15`   | `0.4..0.6`  |
| `0.14`   | `0.3`       |

## License

`bevy_lit` is licensed under the MIT License. See [LICENSE](LICENSE) for more details.
