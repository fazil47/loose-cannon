use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // planet
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 20.0,
            subdivisions: 20,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.0, 20.0),
        ..default()
    });

    // light
    const HALF_SIZE: f32 = 20.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_scaled_axis(Vec3::new(-0.5, 0.5, 0.0)),
            ..default()
        },
        ..default()
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
