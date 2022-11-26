use std::f32::consts::PI;

use bevy::{prelude::*, render::texture::CompressedImageFormats};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

use loose_cannon::cubemap::{construct_skybox, Cubemap, CubemapMaterial};

// TODO: Refactor
// TODO: Restrict player collider altitude
// TODO: game over when a cannon ball hits the player
// TODO: add a single type of enemy
// TODO: enemies should spawn from reasonably spaced random points
// TODO: game over ui
// TODO: count score (in proportion to number of enemies killed)
// TODO: show score in ui
// TODO: cannon ball explosion vfx
// TODO: cannon ball shooting vfx
// TODO: cannon ball explosion sfx
// TODO: cannon ball shooting sfx
// TODO: add grass to planet
// TODO: add trees to planet
// TODO: add mesh for enemy spawn point

const CUBEMAP: &(&str, CompressedImageFormats) = &(
    "textures/skybox/corona_skybox.png",
    CompressedImageFormats::NONE,
);
const PLANET_SIZE: f32 = 20.0;
const PLAYER_SIZE: f32 = 1.0;
const FIRE_DELAY: f32 = 2.0; // Delay in seconds until the next cannon can be fired
const CAMERA_DISTANCE: f32 = 60.0;
const GRAVITY_MAGNITUDE: f32 = 3.0;
const PLAYER_IMPULSE_MAGNITUDE: f32 = 200.0;
const CANNON_BALL_INITIAL_OFFSET: f32 = 3.0;
const SHOW_DEBUG_LINES: bool = false;

#[derive(Component)]
struct PlayerMesh {}
#[derive(Component)]
struct PlayerCollider {}

#[derive(Component)]
struct CannonBall {}

#[derive(Resource)]
struct ShootTimer(Timer);

#[derive(Resource)]
struct PlayerInput {
    last_valid_cursor_pos: Option<Vec2>,
}

#[derive(Resource)]
struct PlayerMeshDesiredTransform {
    position: Vec3,
    tangent: Vec3,
    local_up: Vec3,
    local_forward: Vec3,
}

struct ShootEvent {
    position: Vec3,
    direction: Vec3,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_startup_system(setup)
        .add_system(gravity)
        .add_system(handle_player_input)
        .add_system(set_player_mesh_transform)
        .add_system(apply_player_collider_impulse)
        .add_system(shoot_cannon_ball)
        .add_system(move_camera)
        .add_system(handle_collisions)
        .add_system(construct_skybox)
        .add_event::<ShootEvent>()
        .run();
}

fn setup(
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

// Handles change in cursor position, updates PlayerMeshDesiredTransform resource
// And sends ShootEvent on LMB click based on the ShootTimer resource
fn handle_player_input(
    mut player_input: ResMut<PlayerInput>,
    mut player_mesh_desired_transform: ResMut<PlayerMeshDesiredTransform>,
    mut shoot_timer: ResMut<ShootTimer>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut lines: ResMut<DebugLines>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut ev_shoot: EventWriter<ShootEvent>,
    camera_query: Query<(&GlobalTransform, &Camera)>,
    player_collider_query: Query<&Transform, With<PlayerCollider>>,
) {
    let window: &Window = windows.get_primary().unwrap();
    let (camera_transform, camera) = camera_query.iter().next().unwrap();
    let player_collider_transform = player_collider_query.single();

    player_mesh_desired_transform.position = player_collider_transform.translation;
    player_mesh_desired_transform.local_up = camera_transform.back();
    player_mesh_desired_transform.local_forward = camera_transform.up();

    // If cursor is inside the window
    if let Some(cursor_pos) = window.cursor_position() {
        // Make a raycast from cursor world position parallel to camera direction
        let ray = camera
            .viewport_to_world(camera_transform, cursor_pos)
            .unwrap();
        let max_toi = 600.0;
        let solid = true;
        let filter = QueryFilter::new();

        // If the raycast hits the planet collider
        if let Some((_entity, toi)) =
            rapier_context.cast_ray(ray.origin, ray.direction, max_toi, solid, filter)
        {
            // Set player_input resource
            player_input.last_valid_cursor_pos = Option::Some(cursor_pos);

            // Get the point on the planet where the raycast hit
            let hit_point = ray.origin + (ray.direction * toi);

            let tangent = get_tangent_helper(hit_point, player_collider_transform);

            player_mesh_desired_transform.tangent = tangent;

            // the player can shoot only after the timer is up
            if !shoot_timer.0.tick(time.delta()).finished() {
                return;
            }

            // If the left mouse button is pressed, apply an impulse in the direction of the tangent
            if buttons.just_pressed(MouseButton::Left) {
                if SHOW_DEBUG_LINES {
                    lines.line(ray.origin, hit_point, 20.0);
                }

                shoot_timer.0.reset();

                ev_shoot.send(ShootEvent {
                    position: player_collider_transform.translation
                        - (tangent * CANNON_BALL_INITIAL_OFFSET),
                    direction: -tangent,
                });
            }
        } else if let Some(cursor_pos) = player_input.last_valid_cursor_pos {
            let ray = camera
                .viewport_to_world(camera_transform, cursor_pos)
                .unwrap();

            // This still needs to be checked because last_valid_cursor_pos won't be valid
            // if the game window's position or size has changed
            if let Some((_entity, toi)) =
                rapier_context.cast_ray(ray.origin, ray.direction, max_toi, solid, filter)
            {
                let hit_point = ray.origin + (ray.direction * toi);

                let tangent = get_tangent_helper(hit_point, player_collider_transform);

                player_mesh_desired_transform.tangent = tangent;
            }
        }
    }
}

// Calculates the tangent in the direction of the vector from the player collider to the hit point on the planet
fn get_tangent_helper(hit_point: Vec3, player_collider_transform: &Transform) -> Vec3 {
    // Get the unit vector in the direction of the vector from the hit point to the player
    let hit_to_player_collider = (hit_point - player_collider_transform.translation).normalize();

    // Cross hit_to_player and player collider's normalized translation vector to get the tangent perpendicular to the desired direction
    let tangent = (hit_to_player_collider.cross(player_collider_transform.translation.normalize()))
        .normalize();

    // Cross again to get the desired direction
    let tangent = tangent.cross(player_collider_transform.translation.normalize());

    tangent
}

// Sets the player mesh's transform based on value of PlayerMeshDesiredTransform resource
fn set_player_mesh_transform(
    mut player_mesh_query: Query<&mut Transform, With<PlayerMesh>>,
    player_mesh_desired_transform: Res<PlayerMeshDesiredTransform>,
) {
    let mut player_mesh_transform = player_mesh_query.single_mut();

    // Player translation should be the same as the player collider translation
    player_mesh_transform.translation =
        player_mesh_desired_transform.position.normalize() * PLANET_SIZE;

    // Rotate player transform such that it's up vector is in the same direction as player's translation vector
    player_mesh_transform.set_down(Vec3::ZERO, player_mesh_desired_transform.local_up);

    let target = player_mesh_transform.translation + player_mesh_desired_transform.tangent;

    // Rotate player transform such that it's looking at the target
    player_mesh_transform.look_at(target, player_mesh_desired_transform.local_up);
}

// Applies an impulse to play collider when a ShootEvent is triggered
fn apply_player_collider_impulse(
    mut player_collider_query: Query<&mut ExternalImpulse, With<PlayerCollider>>,
    mut ev_shoot: EventReader<ShootEvent>,
) {
    let mut player_collider_impulse = player_collider_query.single_mut();

    for ev in ev_shoot.iter() {
        player_collider_impulse.impulse = -ev.direction * PLAYER_IMPULSE_MAGNITUDE;
    }
}

// Spawns and shoots a cannon ball when a ShootEvent is triggered
fn shoot_cannon_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_shoot: EventReader<ShootEvent>,
) {
    for ev in ev_shoot.iter() {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: PLAYER_SIZE / 2.0,
                    subdivisions: 16,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.3, 0.3, 0.3).into(),
                    perceptual_roughness: 0.3,
                    metallic: 0.8,
                    ..default()
                }),
                transform: Transform::from_translation(ev.position),
                ..default()
            })
            .insert(CannonBall {})
            .insert(Collider::ball(PLAYER_SIZE))
            .insert(ActiveEvents::COLLISION_EVENTS)
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
                impulse: ev.direction * PLAYER_IMPULSE_MAGNITUDE,
                torque_impulse: Vec3::new(0.0, 0.0, 0.0),
            });
    }
}

// System to handle collision events
fn handle_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    player_collider_query: Query<Entity, With<PlayerCollider>>,
    cannon_ball_query: Query<Entity, With<CannonBall>>,
) {
    for collsion_event in collision_events.iter() {
        // Check only when collision has started
        if let CollisionEvent::Started(collider, other_collider, _) = collsion_event {
            // If collider has a PlayerCollider component
            if let Ok(_entity) = player_collider_query.get(*collider) {
                // TODO: Set game over
                // commands.entity(*collider).despawn();
                commands.entity(*other_collider).despawn();
            } else if let Ok(entity) = cannon_ball_query.get(*collider) {
                commands.entity(entity).despawn();

                if let Ok(entity) = cannon_ball_query.get(*other_collider) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// Move camera to follow the player
fn move_camera(
    mut camera_transforms: Query<&mut Transform, With<Camera3d>>,
    player_query: Query<&Transform, (With<PlayerCollider>, Without<Camera3d>)>,
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
fn gravity(mut query: Query<(&Transform, &mut ExternalForce)>) {
    for (transform, mut force) in query.iter_mut() {
        let grav_force_magnitude = transform.translation.length().powi(2) * GRAVITY_MAGNITUDE;
        force.force = grav_force_magnitude * -transform.translation.normalize();
    }
}

trait TransformExt {
    /// Rotates this [`Transform`] so that its local negative `Y` direction is toward
    /// `target` and its local negative `Z` direction is toward `forward`.
    fn set_down(&mut self, target: Vec3, forward: Vec3);
}

impl TransformExt for Transform {
    fn set_down(&mut self, target: Vec3, forward: Vec3) {
        let up = Vec3::normalize(self.translation - target);
        let right = up.cross(forward).normalize();
        let forward = right.cross(up).normalize();

        self.rotation = Quat::from_mat3(&Mat3::from_cols(right, up, forward));
    }
}
