use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;

const PLANET_SIZE: f32 = 20.0;
const PLAYER_SIZE: f32 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin)
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_startup_system(setup)
        .add_system(gravity)
        .add_system(player_input)
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
                subdivisions: 20,
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
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: PLAYER_SIZE / 2.0,
                subdivisions: 10,
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 21.5),
            ..default()
        })
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
        transform: Transform::from_xyz(0.0, 0.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn player_input(
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    rapier_context: Res<RapierContext>,
    mut lines: ResMut<DebugLines>,
    mut player_query: Query<(&Transform, &mut ExternalImpulse)>,
    camera_query: Query<(&Transform, &Camera)>,
) {
    let window: &Window = windows.get_primary().unwrap();
    let (camera_transform, camera): (&Transform, &Camera) = camera_query.iter().next().unwrap();

    // If cursor is inside the window
    if let Some(cursor_pos) = window.cursor_position() {
        if buttons.just_pressed(MouseButton::Left) {
            for (transform, mut impulse) in player_query.iter_mut() {
                let (cursor_world_pos, cursor_world_dir) =
                    camera_to_cursor_in_world(window, cursor_pos, camera_transform, &camera);

                // Make a raycast from cursor world position parallet to camera direction
                let ray_origin = cursor_world_pos;
                let ray_dir = cursor_world_dir;
                let max_toi = 600.0;
                let solid = true;
                let filter = QueryFilter::new();

                if let Some((_entity, toi)) =
                    rapier_context.cast_ray(ray_origin, ray_dir, max_toi, solid, filter)
                {
                    // Get the point on the planet where the raycast hit
                    let hit_point = ray_origin + (ray_dir * toi);

                    // Get the unit vector in the direction of the vector from the hit point to the player
                    let hit_to_player_dir = (transform.translation - hit_point).normalize();

                    // let angle between hit_to_player_dir and normal on the planet at the player's position is theta
                    let angle = transform
                        .translation
                        .normalize()
                        .dot(hit_to_player_dir)
                        .acos();

                    // then sin(theta) gives the tangent along the planet's surface in the direction of the vector from the hit point to the player
                    let tangent = (hit_to_player_dir * angle.sin()).normalize();

                    lines.line(ray_origin, hit_point, 20.0);

                    impulse.impulse = hit_to_player_dir.normalize() * 100.0;
                } else {
                    lines.line_colored(
                        ray_origin,
                        ray_origin + (ray_dir * max_toi),
                        20.0,
                        Color::RED,
                    );
                }
            }
        }
    }
}

// Custom gravity which acts towards the center of the planet (which is at the origin)
fn gravity(mut query: Query<(&Transform, &mut ExternalForce)>) {
    for (transform, mut force) in query.iter_mut() {
        force.force = transform.translation.normalize_or_zero() * -9.81 * 100.0;
    }
}

// TODO: Remove this when bevy 0.9 is released
// Cast a ray from the camera to the cursor in world space
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
