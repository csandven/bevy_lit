use bevy::{
    color::palettes::tailwind::{AMBER_200, BLUE_300, GRAY_300, GRAY_700, SLATE_700},
    prelude::*,
};
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .insert_resource(ClearColor(Color::from(GRAY_300)))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sun)
        .run();
}

#[derive(Component)]
struct Sun;

const X_EXTENT: f32 = 700.0;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Lighting2dSettings {
            blur: 4,
            edge_intensity: 8.0,
            ..default()
        },
        AmbientLight2d {
            intensity: 0.18,
            color: Color::from(BLUE_300),
        },
    ));

    commands.spawn((
        Sun,
        DirectionalLight2d {
            color: Color::from(AMBER_200),
            strength: 0.6,
            direction: Vec2::new(0.8, 1.0).normalize(),
            tile_size: 64.0,
        },
    ));

    let shapes = [
        meshes.add(Rectangle::new(70.0, 70.0)),
        meshes.add(Circle::new(38.0)),
        meshes.add(Capsule2d::new(22.0, 60.0)),
        meshes.add(RegularPolygon::new(45.0, 6)),
        meshes.add(Triangle2d::new(
            Vec2::new(0.0, 52.0),
            Vec2::new(-42.0, -42.0),
            Vec2::new(42.0, -42.0),
        )),
    ];

    let occluder_material = materials.add(Color::from(GRAY_700));
    let count = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh2d(shape),
            MeshMaterial2d(occluder_material.clone()),
            LightOccluder2d {
                height: 8.0 + (i as f32 * 6.0),
                ..default()
            },
            Transform::from_xyz(
                -X_EXTENT / 2.0 + i as f32 / (count - 1) as f32 * X_EXTENT,
                -40.0,
                0.0,
            ),
        ));
    }

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(280.0, 170.0))),
        MeshMaterial2d(materials.add(Color::from(SLATE_700))),
        Transform::from_xyz(250.0, 130.0, 2.0),
    ));

    // Suppress directional shadows under the roofed region.
    commands.spawn((
        RoofMask2d::new(Vec2::new(280.0, 170.0)),
        Transform::from_xyz(250.0, 130.0, 0.0),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(90.0, 90.0))),
        MeshMaterial2d(occluder_material),
        LightOccluder2d {
            height: 20.0,
            ..default()
        },
        Transform::from_xyz(250.0, 130.0, 1.0),
    ));
}

fn animate_sun(time: Res<Time>, mut sun: Single<&mut DirectionalLight2d, With<Sun>>) {
    let phase = time.elapsed_secs() * 0.35;
    let x = phase.cos();
    let y = 0.35 + phase.sin().abs() * 0.9;

    sun.direction = Vec2::new(x, y).normalize_or_zero();
    sun.strength = (0.25 + y * 0.55).clamp(0.2, 1.0);
}
