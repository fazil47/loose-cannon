use bevy::prelude::{Component, EventReader, Query, Res, Resource, Transform, Vec3, With};
use bevy_rapier3d::prelude::ExternalImpulse;

use crate::{
    constants::{PLANET_SIZE, PLAYER_IMPULSE_MAGNITUDE},
    extensions::TransformExt,
    input::ShootEvent,
};

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
    mut player_collider_query: Query<&mut ExternalImpulse, With<PlayerCollider>>,
    mut ev_shoot: EventReader<ShootEvent>,
) {
    let mut player_collider_impulse = player_collider_query.single_mut();

    for ev in ev_shoot.iter() {
        player_collider_impulse.impulse = -ev.direction * PLAYER_IMPULSE_MAGNITUDE;
    }
}
