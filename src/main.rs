use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin)
        .add_startup_system(setup)
        .add_system(gravity)
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
                radius: 20.0,
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
        .insert(Restitution::coefficient(0.0));

    // player
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 1.0,
                subdivisions: 10,
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.0, 22.5),
            ..default()
        })
        .insert(Collider::ball(1.0))
        .insert(RigidBody::Dynamic)
        .insert(ColliderMassProperties::Density(2.0))
        .insert(GravityScale(0.0))
        .insert(Friction {
            coefficient: 2.0,
            combine_rule: CoefficientCombineRule::Max,
        })
        .insert(Restitution::coefficient(0.0))
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

// Custom gravity which acts towards the center of the planet (which is at the origin)
fn gravity(mut query: Query<(&Transform, &mut ExternalForce)>) {
    for (transform, mut force) in query.iter_mut() {
        force.force = transform.translation.normalize_or_zero() * -9.81 * 10.0;
    }
}
