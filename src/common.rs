use bevy::{
    prelude::{
        default, shape, AmbientLight, Assets, Camera, Camera3dBundle, Color, Commands, Component,
        DespawnRecursiveExt, DirectionalLight, DirectionalLightBundle, Entity, EventReader, Mesh,
        Name, NonSend, OrthographicProjection, PbrBundle, Quat, Query, ResMut, Resource,
        StandardMaterial, State, Transform, Vec3, With, Without,
    },
    window::WindowId,
    winit::WinitWindows,
};
use bevy_atmosphere::prelude::{AtmosphereCamera, AtmosphereModel, Gradient};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, CollisionEvent, ExternalForce, Friction,
    RapierColliderHandle, RapierContext, RapierRigidBodyHandle, Restitution,
};
use image;
use std::f32::consts::PI;
use winit::window::Icon;

use crate::{asteroid::Asteroid, cannon_ball::CannonBall, player::PlayerCollider};

// CONSTANTS
pub const SHOW_DEBUG_LINES: bool = false;
pub const PLANET_SIZE: f32 = 20.0;
pub const CAMERA_DISTANCE: f32 = 60.0;
pub const GRAVITY_MAGNITUDE: f32 = 3.0;
pub const SCORE_INCREMENT: i32 = 1;

// COMPONENTS

#[derive(Component)]
pub struct PrimaryCamera {}

// RESOURCES

#[derive(Resource)]
pub struct Score(pub i32);

// STATES

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Playing,
    GameOver,
}

// STARTUP SYSTEMS

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Scene Camera
    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    priority: 10,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, CAMERA_DISTANCE)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            AtmosphereCamera::default(),
        ))
        .insert(PrimaryCamera {});

    // Skybox
    commands.insert_resource(AtmosphereModel::new(Gradient {
        sky: Color::WHITE,
        horizon: Color::SALMON,
        ground: Color::ORANGE_RED,
    }));

    // Planet
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: PLANET_SIZE,
                subdivisions: 32,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.5, 0.3),
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

    // Directional light - sun
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::Rgba {
                    red: 1.0,
                    green: 0.7,
                    blue: 0.7,
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
                translation: Vec3::new(0.0, PLANET_SIZE, 0.0),
                rotation: Quat::from_rotation_x(-PI / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Sun"));

    // Directional light - reflected sun
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 0.5,
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
                translation: Vec3::new(0.0, -PLANET_SIZE, 0.0),
                rotation: Quat::from_rotation_x(PI / 2.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Reflected Sun"));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(1.0, 1.0, 0.8),
        brightness: 0.4,
    });
}

pub fn reset_rapier(
    mut commands: Commands,
    mut rapier: ResMut<RapierContext>,
    collider_handles: Query<Entity, With<RapierColliderHandle>>,
    rb_handles: Query<Entity, With<RapierRigidBodyHandle>>,
) {
    // Force rapier to reload everything
    for e in collider_handles.iter() {
        commands.entity(e).remove::<RapierColliderHandle>();
    }
    for e in rb_handles.iter() {
        commands.entity(e).remove::<RapierRigidBodyHandle>();
    }

    // Re-initialize everything we overwrite with default values
    let context = RapierContext::default();
    rapier.bodies = context.bodies;
    rapier.colliders = context.colliders;
    rapier.broad_phase = context.broad_phase;
    rapier.narrow_phase = context.narrow_phase;
    rapier.ccd_solver = context.ccd_solver;
    rapier.impulse_joints = context.impulse_joints;
    rapier.integration_parameters = context.integration_parameters;
    rapier.islands = context.islands;
    rapier.multibody_joints = context.multibody_joints;
    rapier.pipeline = context.pipeline;
    rapier.query_pipeline = context.query_pipeline;
}

pub fn setup_window(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();

    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("assets/icons/main_icon.png")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary.set_window_icon(Some(icon));
}

// SYSTEMS

// System to handle collision events
pub fn handle_collisions(
    mut commands: Commands,
    mut game_state: ResMut<State<GameState>>,
    mut score: ResMut<Score>,
    mut ev_collision: EventReader<CollisionEvent>,
    player_collider_query: Query<Entity, With<PlayerCollider>>,
    cannon_ball_query: Query<Entity, With<CannonBall>>,
    asteroid_query: Query<Entity, With<Asteroid>>,
) {
    for collsion_event in ev_collision.iter() {
        // Check only when collision has started
        if let CollisionEvent::Started(collider, other_collider, _) = collsion_event {
            if player_collider_query.get(*collider).is_ok()
                || player_collider_query.get(*other_collider).is_ok()
            {
                if let Err(error) = game_state.overwrite_replace(GameState::GameOver) {
                    println!("Error setting game state to GameOver: {}", error);
                }
            } else if cannon_ball_query.get(*collider).is_ok() {
                if asteroid_query.get(*other_collider).is_ok() {
                    score.0 += SCORE_INCREMENT;
                    commands.entity(*collider).despawn();
                    commands.entity(*other_collider).despawn();
                } else if cannon_ball_query.get(*other_collider).is_ok() {
                    commands.entity(*collider).despawn();
                    commands.entity(*other_collider).despawn();
                }
            } else if asteroid_query.get(*collider).is_ok() {
                if cannon_ball_query.get(*other_collider).is_ok() {
                    score.0 += SCORE_INCREMENT;
                    commands.entity(*collider).despawn();
                    commands.entity(*other_collider).despawn();
                } else if asteroid_query.get(*other_collider).is_ok() {
                    commands.entity(*collider).despawn();
                    commands.entity(*other_collider).despawn();
                }
            }
        }
    }
}

// Move primary camera to follow the player
pub fn move_camera(
    mut camera_transforms: Query<&mut Transform, With<PrimaryCamera>>,
    player_query: Query<&Transform, (With<PlayerCollider>, Without<PrimaryCamera>)>,
) {
    let mut camera_transform = camera_transforms.iter_mut().next().unwrap();

    let player_transform = player_query.single();
    let player_translation_scaled = player_transform.translation.normalize() * CAMERA_DISTANCE;

    if camera_transform
        .translation
        .distance(player_translation_scaled)
        > 0.1
    {
        let new_camera_translation = camera_transform
            .translation
            .lerp(player_translation_scaled, 0.4);

        *camera_transform = Transform::from_translation(new_camera_translation)
            .looking_at(Vec3::ZERO, camera_transform.up());
    }
}

// Custom gravity which acts towards the center of the planet (which is at the origin)
pub fn gravity(mut query: Query<(&Transform, &mut ExternalForce)>) {
    for (transform, mut force) in query.iter_mut() {
        let grav_force_magnitude = transform.translation.length().powi(2) * GRAVITY_MAGNITUDE;
        force.force = grav_force_magnitude * -transform.translation.normalize();
    }
}

// CLEANUP SYSTEMS

// Remove all entities except cameras plus the primary camera
pub fn teardown(
    mut commands: Commands,
    entities: Query<Entity, Without<Camera>>,
    primary_camera_query: Query<Entity, With<PrimaryCamera>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in primary_camera_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// Reset score to 0
pub fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}
