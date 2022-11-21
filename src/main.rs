use std::f32::consts::PI;

use bevy::{prelude::*, render::texture::CompressedImageFormats};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

use loose_cannon::cubemap::{construct_skybox, Cubemap, CubemapMaterial};

const CUBEMAP: &(&str, CompressedImageFormats) = &(
    "textures/skybox/corona_skybox.png",
    CompressedImageFormats::NONE,
);
const PLANET_SIZE: f32 = 20.0;
const PLAYER_SIZE: f32 = 1.0;
const CAMERA_DISTANCE: f32 = 60.0;
const GRAVITY_MAGNITUDE: f32 = 3.0;
const PLAYER_IMPULSE_MAGNITUDE: f32 = 200.0;
const SHOW_DEBUG_LINES: bool = false;

#[derive(Component)]
struct PlayerMesh {}
#[derive(Component)]
struct PlayerCollider {}

#[derive(Component)]
struct CannonBall {}

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
        .add_system(player_input)
        .add_system(move_camera)
        .add_system(construct_skybox)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // planet
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

    // player mesh
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/cannon.glb#Scene0"),
            transform: Transform::from_scale(Vec3::new(0.25, 0.25, 0.25)),
            ..default()
        })
        .insert(Name::new("PlayerMesh"))
        .insert(PlayerMesh {});

    // player collider
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

    // directional light - sun
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::Rgba {
                    red: 1.0,
                    green: 0.7,
                    blue: 0.5,
                    alpha: 1.0,
                },
                illuminance: 100_000.0,
                shadow_projection: OrthographicProjection {
                    left: -PLANET_SIZE,
                    right: PLANET_SIZE,
                    bottom: -PLANET_SIZE,
                    top: PLANET_SIZE,
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

    // directional light - moon
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
                    left: -PLANET_SIZE,
                    right: PLANET_SIZE,
                    bottom: -PLANET_SIZE,
                    top: PLANET_SIZE,
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

    // camera
    // Querying doesn't work if I name the camera entity
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, CAMERA_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(1.0, 1.0, 0.8),
        brightness: 0.4,
    });

    // skybox
    let skybox_handle = asset_server.load(CUBEMAP.0);
    commands.insert_resource(Cubemap {
        image_handle: skybox_handle,
        is_loaded: false,
    });
}

// Handle player input
fn player_input(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    mut lines: ResMut<DebugLines>,
    mut player_collider_query: Query<
        (&Transform, &mut ExternalImpulse),
        (With<PlayerCollider>, Without<PlayerMesh>),
    >,
    mut player_mesh_query: Query<&mut Transform, (With<PlayerMesh>, Without<PlayerCollider>)>,
    camera_query: Query<
        (&GlobalTransform, &Camera),
        (Without<ExternalImpulse>, Without<PlayerMesh>),
    >,
) {
    let window: &Window = windows.get_primary().unwrap();
    let (camera_transform, camera) = camera_query.iter().next().unwrap();
    let (player_c_transform, mut player_c_impulse) = player_collider_query.single_mut();
    let mut player_mesh_transform = player_mesh_query.single_mut();

    // Player translation should be the same as the player collider translation
    player_mesh_transform.translation = player_c_transform.translation.normalize() * PLANET_SIZE;

    // Rotate player transform such that it's up vector is in the same direction as player's translation vector
    player_mesh_transform.set_down(Vec3::ZERO, camera_transform.up());

    // If cursor is inside the window
    if let Some(cursor_pos) = window.cursor_position() {
        // Make a raycast from cursor world position parallet to camera direction
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
            // Get the point on the planet where the raycast hit
            let hit_point = ray.origin + (ray.direction * toi);

            // Get the unit vector in the direction of the vector from the hit point to the player
            let hit_to_player = (hit_point - player_c_transform.translation).normalize();

            // Cross hit_to_player and player collider's normalized translation vector to get the tangent perpendicular to the desired direction
            let tangent =
                (hit_to_player.cross(player_c_transform.translation.normalize())).normalize();

            // Cross again to get the desired direction
            let tangent = tangent.cross(player_c_transform.translation.normalize());

            if SHOW_DEBUG_LINES {
                lines.line_colored(
                    player_c_transform.translation,
                    player_c_transform.translation + tangent,
                    1.0,
                    Color::GREEN,
                );
            }

            let target = player_mesh_transform.translation + tangent;

            // player_transform.rotate_local_y(player_mesh_angle);
            player_mesh_transform.look_at(target, camera_transform.back());

            // If the left mouse button is pressed, apply an impulse in the direction of the tangent
            if buttons.just_pressed(MouseButton::Left) {
                if SHOW_DEBUG_LINES {
                    lines.line(ray.origin, hit_point, 20.0);
                }

                player_c_impulse.impulse = tangent * PLAYER_IMPULSE_MAGNITUDE;

                // shoot cannon ball
                shoot_cannon_ball(
                    commands,
                    meshes,
                    materials,
                    player_mesh_transform.translation - tangent,
                    -tangent,
                )
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

// Shoot cannon ball helper function
fn shoot_cannon_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    direction: Vec3,
) {
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
            transform: Transform::from_translation(position),
            ..default()
        })
        .insert(CannonBall {})
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
            impulse: direction * PLAYER_IMPULSE_MAGNITUDE,
            torque_impulse: Vec3::new(0.0, 0.0, 0.0),
        });
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
