use bevy::{math::vec3, prelude::*};
use bevy::window::PrimaryWindow;
use bevy_lit::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, Lighting2dPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update_cursor_light)
        .add_systems(FixedUpdate, update_moving_lights)
        .run();
}

#[derive(Component)]
struct CursorLight;

#[derive(Component)]
struct MovingLights;

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Lighting2dSettings {
            blur: 32.0,
            raymarch: RaymarchSettings {
                max_steps: 32,
                jitter_contrib: 0.5,
                sharpness: 10.0,
            },
            ..default()
        },
        AmbientLight2d {
            brightness: 0.2,
            color: Color::Srgba(Srgba::hex("#C09AFE").unwrap()),
        },
    ));

    vec![
        vec3(-150.0, 0.0, 0.0),
        vec3(0.0, -150.0, 0.0),
        vec3(150.0, 0.0, 0.0),
        vec3(0.0, 150.0, 0.0),
    ]
    .into_iter()
    .for_each(|pos| {
        commands.spawn((
            Sprite {
                custom_size: Some(Vec2::splat(100.0)),
                ..default()
            },
            Transform::from_translation(pos),
            LightOccluder2d {
                half_size: Vec2::splat(50.0),
            },
        ));
    });

    commands
        .spawn((MovingLights, Transform::default(), Visibility::default()))
        .with_children(|builder| {
            let point_light = PointLight2d {
                intensity: 3.0,
                radius: 1000.0,
                falloff: 3.0,
                ..default()
            };

            builder.spawn((
                PointLight2d {
                    color: Color::srgb(0.0, 1.0, 1.0),
                    ..point_light
                },
                Transform::from_xyz(-500.0, 0.0, 0.0)
            ));

            builder.spawn((
                PointLight2d {
                    color: Color::srgb(1.0, 0.0, 1.0),
                    ..point_light
                },
                Transform::from_xyz(500.0, 0.0, 0.0)
            ));
        });

    commands.spawn((
        CursorLight,
        PointLight2d {
            intensity: 4.0,
            radius: 400.0,
            falloff: 3.0,
            color: Color::srgb(1.0, 1.0, 0.0),
        },
    ));
}

fn update_cursor_light(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut point_light_query: Query<&mut Transform, With<CursorLight>>,
) {
    let (camera, camera_transform) = camera_query.single();
    let window = window_query.single();
    let mut point_light_transform = point_light_query.single_mut();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate().extend(0.0))
    {
        point_light_transform.translation = world_position;
    }
}

fn update_moving_lights(
    time: Res<Time>,
    mut point_light_query: Query<&mut Transform, With<MovingLights>>,
) {
    for mut transform in &mut point_light_query {
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() / 4.0);
    }
}
