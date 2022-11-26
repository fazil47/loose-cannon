use std::f32::consts::PI;

use bevy::{
    prelude::{
        default, shape, AmbientLight, AssetServer, Assets, Camera3dBundle, Color, Commands,
        DirectionalLight, DirectionalLightBundle, Mesh, Name, OrthographicProjection, PbrBundle,
        Quat, Res, ResMut, StandardMaterial, Transform, Vec3,
    },
    scene::SceneBundle,
    time::{Timer, TimerMode},
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ColliderMassProperties, Damping, ExternalForce,
    ExternalImpulse, Friction, GravityScale, Restitution, RigidBody,
};

use crate::{
    constants::{CAMERA_DISTANCE, CUBEMAP, FIRE_DELAY, PLANET_SIZE, PLAYER_SIZE},
    cubemap::Cubemap,
    input::{PlayerInput, ShootTimer},
    player::{PlayerCollider, PlayerMesh, PlayerMeshDesiredTransform},
};

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Insert resource to keep track of player cursor position
    commands.insert_resource(PlayerInput {
        last_valid_cursor_pos: Option::None,
    });

    // Insert resouce to keep track of time until the next cannon ball can be fired
    let mut timer = Timer::from_seconds(FIRE_DELAY, TimerMode::Once);
    timer.tick(timer.duration());
    commands.insert_resource(ShootTimer(timer));

    // Planet
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: PLANET_SIZE,
                subdivisions: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.5, 0.3).into(),
                perceptual_roughness: 0.8,
                metallic: 0.4,
                ..default()
            }),
            ..default()
        })
        .insert(Name::new("Planet"))
        .insert(Collider::ball(20.0))
        .insert(Friction {
            coefficient: 2.0,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Max,
        });

    // Player mesh
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/cannon.glb#Scene0"),
            transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25)),
            ..default()
        })
        .insert(Name::new("PlayerMesh"))
        .insert(PlayerMesh {});

    // Resource to store desired transform of player mesh
    commands.insert_resource(PlayerMeshDesiredTransform {
        position: Vec3::new(0.0, 0.0, PLANET_SIZE + PLAYER_SIZE),
        tangent: Vec3::new(0.0, 1.0, 0.0),
        local_up: Vec3::new(0.0, 0.0, 1.0),
        local_forward: Vec3::new(0.0, 1.0, 0.0),
    });

    // Player collider
    commands
        .spawn(TransformBundle::from(Transform::from_xyz(
            0.0,
            0.0,
            PLANET_SIZE + PLAYER_SIZE,
        )))
        .insert(Name::new("PlayerCollider"))
        .insert(PlayerCollider {})
        .insert(Collider::ball(PLAYER_SIZE))
        .insert(RigidBody::Dynamic)
        .insert(Damping {
            linear_damping: 0.1,
            angular_damping: 0.2,
        })
        .insert(ColliderMassProperties::Density(1.0))
        .insert(GravityScale(0.0))
        .insert(Friction {
            coefficient: 2.0,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(ExternalForce {
            force: Vec3::new(0.0, 0.0, 0.0),
            torque: Vec3::new(0.0, 0.0, 0.0),
        })
        .insert(ExternalImpulse {
            impulse: Vec3::new(0.0, 0.0, 0.0),
            torque_impulse: Vec3::new(0.0, 0.0, 0.0),
        });

    // Directional light - sun
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::Rgba {
                    red: 0.9,
                    green: 0.7,
                    blue: 1.0,
                    alpha: 1.0,
                },
                illuminance: 100_000.0,
                shadow_projection: OrthographicProjection {
                    left: -2.0 * PLANET_SIZE,
                    right: 2.0 * PLANET_SIZE,
                    bottom: -2.0 * PLANET_SIZE,
                    top: 2.0 * PLANET_SIZE,
                    near: -10.0 * PLANET_SIZE,
                    far: 10.0 * PLANET_SIZE,
                    ..default()
                },
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_y(PI / 3.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Sun"));

    // Directional light - moon
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::Rgba {
                    red: 0.7,
                    green: 0.7,
                    blue: 1.0,
                    alpha: 1.0,
                },
                illuminance: 10_000.0,
                shadow_projection: OrthographicProjection {
                    left: -2.0 * PLANET_SIZE,
                    right: 2.0 * PLANET_SIZE,
                    bottom: -2.0 * PLANET_SIZE,
                    top: 2.0 * PLANET_SIZE,
                    near: -10.0 * PLANET_SIZE,
                    far: 10.0 * PLANET_SIZE,
                    ..default()
                },
                shadows_enabled: true,
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_y(4.0 * PI / 3.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Moon"));

    // Camera
    // Querying doesn't work if I name the camera entity
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, CAMERA_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(1.0, 1.0, 0.8),
        brightness: 0.4,
    });

    // Skybox
    let skybox_handle = asset_server.load(CUBEMAP.0);
    commands.insert_resource(Cubemap {
        image_handle: skybox_handle,
        is_loaded: false,
    });
}
