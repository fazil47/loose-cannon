use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

const PLANET_SIZE: f32 = 20.0;
const PLAYER_SIZE: f32 = 2.0;
const CAMERA_DISTANCE: f32 = 60.0;
const GRAVITY_MAGNITUDE: f32 = 3.0;
const PLAYER_IMPULSE_MAGNITUDE: f32 = 200.0;

#[derive(Component)]
struct Player {}
#[derive(Component)]
struct PlayerCollider {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin)
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_startup_system(setup)
        .add_system(gravity)
        .add_system(update_player_transform)
        .add_system(player_input)
        .add_system(move_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // planet
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: PLANET_SIZE,
                subdivisions: 32,
            })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(Collider::ball(20.0))
        .insert(Friction {
            coefficient: 2.0,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Restitution {
            coefficient: 0.0,
            combine_rule: CoefficientCombineRule::Max,
        });

    // player
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: PLAYER_SIZE })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 21.5),
            ..default()
        })
        .insert(Player {});

    // player collider
    commands
        .spawn_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 21.5)))
        .insert(Collider::ball(1.0))
        .insert(PlayerCollider {})
        .insert(Collider::ball(1.0))
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

    // directional light
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

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(1.0, 1.0, 0.8),
        brightness: 0.1,
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 0.0, CAMERA_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn update_player_transform(
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCollider>)>,
    player_collider_query: Query<&Transform, (With<PlayerCollider>, Without<Player>)>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
) {
    let mut player_transform = player_query.single_mut();
    let player_collider_transform = player_collider_query.single();
    let camera_transform = camera_query.iter().next().unwrap();

    // Player translation is the same as the player collider translation
    player_transform.translation = player_collider_transform.translation;

    // Rotate player transform such that it's up vector is in the same direction as player's translation vector
    player_transform.set_down(Vec3::ZERO, camera_transform.up());
}

// Handle player input
fn player_input(
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    mut lines: ResMut<DebugLines>,
    mut player_collider_query: Query<
        (&Transform, &mut ExternalImpulse),
        (With<PlayerCollider>, Without<Player>),
    >,
    mut player_query: Query<&mut Transform, (With<Player>, Without<PlayerCollider>)>,
    camera_query: Query<(&Transform, &Camera), (Without<ExternalImpulse>, Without<Player>)>,
) {
    let window: &Window = windows.get_primary().unwrap();
    let (camera_transform, camera) = camera_query.iter().next().unwrap();
    let (player_c_transform, mut player_c_impulse) = player_collider_query.single_mut();
    let mut player_transform = player_query.single_mut();

    // If cursor is inside the window
    if let Some(cursor_pos) = window.cursor_position() {
        let (cursor_world_pos, cursor_world_dir) =
            camera_to_cursor_in_world(window, cursor_pos, camera_transform, &camera);

        // Make a raycast from cursor world position parallet to camera direction
        let ray_origin = cursor_world_pos;
        let ray_dir = cursor_world_dir;
        let max_toi = 600.0;
        let solid = true;
        let filter = QueryFilter::new();

        // If the raycast hits the planet collider
        if let Some((_entity, toi)) =
            rapier_context.cast_ray(ray_origin, ray_dir, max_toi, solid, filter)
        {
            // Get the point on the planet where the raycast hit
            let hit_point = ray_origin + (ray_dir * toi);

            // Scaled such that hit point is at the same distance from the planet as the player
            let hit_point_scaled = hit_point.normalize() * (PLANET_SIZE + PLAYER_SIZE / 2.0);

            // Get the unit vector in the direction of the vector from the hit point to the player
            let hit_to_player_dir = (player_c_transform.translation - hit_point_scaled).normalize();

            // let angle between hit_to_player_dir and normal on the planet at the player's position be theta
            let angle = player_c_transform
                .translation
                .normalize()
                .dot(hit_to_player_dir)
                .acos();

            // then sin(theta) gives the tangent along the planet's surface in the direction of the vector from the hit point to the player
            let tangent = (hit_to_player_dir * angle.sin()).normalize();

            // Rotate player mesh such that it's +y axis is aligned with the tangent
            let player_mesh_angle = tangent.angle_between(player_transform.forward());
            player_transform.rotate_local_y(player_mesh_angle);

            // If the left mouse button is pressed, apply an impulse in the direction of the tangent
            if buttons.just_pressed(MouseButton::Left) {
                lines.line(ray_origin, hit_point_scaled, 20.0);
                lines.line_colored(
                    player_c_transform.translation,
                    player_c_transform.translation + tangent,
                    20.0,
                    Color::GREEN,
                );

                player_c_impulse.impulse = tangent * PLAYER_IMPULSE_MAGNITUDE;
            }
        }
    }
}

// Move camera to follow the player
fn move_camera(
    mut camera_transforms: Query<(&mut Transform, &Camera3d)>,
    player_query: Query<&Transform, (With<PlayerCollider>, Without<Camera3d>)>,
) {
    let (mut camera_transform, _camera) = camera_transforms.iter_mut().next().unwrap();

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

// TODO: Remove this when bevy 0.9 is released
// Returns a ray from the camera to the cursor's position in world space
// Returns origin of the ray and direction of the ray in world space
pub fn camera_to_cursor_in_world(
    primary_window: &Window,
    cursor_pos: Vec2,
    camera_transform: &Transform,
    camera: &Camera,
) -> (Vec3, Vec3) {
    let ndc = (cursor_pos / Vec2::new(primary_window.width(), primary_window.height())
        - Vec2::new(0.5, 0.5))
        * Vec2::new(2.0, 2.0);
    let point_1 = ndc.extend(1.);
    let point_2 = ndc.extend(0.5);

    let point_1 =
        camera_transform.mul_vec3(camera.projection_matrix().inverse().project_point3(point_1));
    let point_2 =
        camera_transform.mul_vec3(camera.projection_matrix().inverse().project_point3(point_2));

    (point_1, point_2 - point_1)
}

// TODO: Check if the rotation is correct
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
