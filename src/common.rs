use bevy::{
    prelude::{
        Camera, Camera3d, Commands, DespawnRecursiveExt, Entity, EventReader, NonSend, Query,
        ResMut, Resource, State, Transform, Vec3, With, Without,
    },
    window::WindowId,
    winit::WinitWindows,
};
use bevy_rapier3d::prelude::{CollisionEvent, ExternalForce, RigidBody, Sleeping};
use image;
use winit::window::Icon;

use crate::{
    cannon_ball::CannonBall,
    constants::{CAMERA_DISTANCE, GRAVITY_MAGNITUDE},
    player::PlayerCollider,
};

// STATES

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    Playing,
    GameOver,
}

// RESOURCES

#[derive(Resource)]
pub struct Score(pub u32);

// STARTUP SYSTEMS

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
    mut ev_collision: EventReader<CollisionEvent>,
    player_collider_query: Query<Entity, With<PlayerCollider>>,
    cannon_ball_query: Query<Entity, With<CannonBall>>,
    mut sleepable_rigidbody_query: Query<&mut Sleeping, With<RigidBody>>,
) {
    for collsion_event in ev_collision.iter() {
        // Check only when collision has started
        if let CollisionEvent::Started(collider, other_collider, _) = collsion_event {
            // If collider has a PlayerCollider component
            if let Ok(_entity) = player_collider_query.get(*collider) {
                game_state.overwrite_set(GameState::GameOver).unwrap();

                // Put all rigidbodies to sleep
                for mut rigidbody in sleepable_rigidbody_query.iter_mut() {
                    rigidbody.sleeping = true;
                }

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
        let grav_force_magnitude = transform.translation.length().powi(3) * GRAVITY_MAGNITUDE;
        force.force = grav_force_magnitude * -transform.translation.normalize();
    }
}

// Remove all entities that are not a camera
pub fn teardown(mut commands: Commands, entities: Query<Entity, Without<Camera>>) {
    for entity in &entities {
        commands.entity(entity).despawn_recursive();
    }
}
