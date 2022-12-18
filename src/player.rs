use bevy::{
    prelude::{
        default, AssetServer, Commands, Component, EventReader, Name, Query, Res, Resource,
        Transform, Vec3, With,
    },
    scene::SceneBundle,
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule, Collider, ColliderMassProperties, Damping, ExternalForce,
    ExternalImpulse, Friction, GravityScale, Restitution, RigidBody, Velocity,
};

use crate::{common::PLANET_SIZE, extensions::TransformExt, input::ShootEvent};

// CONSTANTS

pub const PLAYER_SIZE: f32 = 1.0;
pub const FIRE_DELAY: f32 = 0.5; // Delay in seconds until the next cannon can be fired
pub const PLAYER_IMPULSE_MAGNITUDE: f32 = 200.0;

// COMPONENTS

#[derive(Component)]
pub struct PlayerMesh {}

#[derive(Component)]
pub struct PlayerCollider {}

// RESOURCES

#[derive(Resource)]
pub struct PlayerMeshDesiredTransform {
    pub position: Vec3,
    pub tangent: Vec3,
    pub local_up: Vec3,
    pub local_forward: Vec3,
}

// STARTUP SYSTEMS

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Resource to store desired transform of player mesh
    commands.insert_resource(PlayerMeshDesiredTransform {
        position: Vec3::new(0.0, 0.0, PLANET_SIZE + PLAYER_SIZE),
        tangent: Vec3::new(0.0, 1.0, 0.0),
        local_up: Vec3::new(0.0, 0.0, 1.0),
        local_forward: Vec3::new(0.0, 1.0, 0.0),
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
        .insert(Velocity {
            linvel: Vec3::ZERO,
            angvel: Vec3::ZERO,
        })
        .insert(ExternalForce {
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
        })
        .insert(ExternalImpulse {
            impulse: Vec3::ZERO,
            torque_impulse: Vec3::ZERO,
        });
}

// SYSTEMS

// Sets the player mesh's transform based on value of PlayerMeshDesiredTransform resource
pub fn set_player_mesh_transform(
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
pub fn apply_player_collider_impulse(
    mut player_collider_query: Query<(&Velocity, &mut ExternalImpulse), With<PlayerCollider>>,
    mut ev_shoot: EventReader<ShootEvent>,
) {
    let (player_collider_velocity, mut player_collider_impulse) =
        player_collider_query.single_mut();

    for ev in ev_shoot.iter() {
        // Apply impulse in the opposite direction of the shoot event
        // the impulse in the direction of the collider's velocity is ignored
        let impulse = -ev.direction * PLAYER_IMPULSE_MAGNITUDE;
        let excess_velocity = player_collider_velocity.linvel;

        player_collider_impulse.impulse = impulse - excess_velocity;
    }
}
