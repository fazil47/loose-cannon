use bevy::prelude::{
    Camera3d, Commands, Entity, EventReader, Query, Transform, Vec3, With, Without,
};
use bevy_rapier3d::prelude::{CollisionEvent, ExternalForce};

use crate::{
    cannon_ball::CannonBall,
    constants::{CAMERA_DISTANCE, GRAVITY_MAGNITUDE},
    player::PlayerCollider,
};

// SYSTEMS

// System to handle collision events
pub fn handle_collisions(
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
pub fn move_camera(
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
pub fn gravity(mut query: Query<(&Transform, &mut ExternalForce)>) {
    for (transform, mut force) in query.iter_mut() {
        let grav_force_magnitude = transform.translation.length().powi(2) * GRAVITY_MAGNITUDE;
        force.force = grav_force_magnitude * -transform.translation.normalize();
    }
}
