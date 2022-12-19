use bevy::{
    prelude::{
        Camera, Commands, EventWriter, GlobalTransform, Input, MouseButton, Query, Res, ResMut,
        Resource, Time, Timer, Transform, Vec2, Vec3, Visibility, Window, Windows, With,
    },
    time::TimerMode,
};
use bevy_prototype_debug_lines::DebugLines;
use bevy_rapier3d::prelude::{QueryFilter, RapierContext};

use crate::{
    cannon_ball::CANNON_BALL_INITIAL_OFFSET,
    common::{PrimaryCamera, SHOW_DEBUG_LINES},
    player::{PlayerCollider, PlayerMeshDesiredTransform, FIRE_DELAY},
    ui::ReloadUI,
};

// RESOURCES

#[derive(Resource)]
pub struct PlayerInput {
    pub last_valid_cursor_pos: Option<Vec2>,
}

#[derive(Resource)]
pub struct ShootTimer(pub Timer);

// EVENTS

pub struct ShootEvent {
    pub position: Vec3,
    pub direction: Vec3,
}

// STARTUP SYSTEMS

pub fn setup_player_input(mut commands: Commands) {
    // Insert resource to keep track of player cursor position
    commands.insert_resource(PlayerInput {
        last_valid_cursor_pos: Option::None,
    });

    // Insert resouce to keep track of time until the next cannon ball can be fired
    let mut timer = Timer::from_seconds(FIRE_DELAY, TimerMode::Once);
    timer.tick(timer.duration());
    commands.insert_resource(ShootTimer(timer));
}

// SYSTEMS

// Handles change in cursor position, updates PlayerMeshDesiredTransform resource
// And sends ShootEvent on LMB click based on the ShootTimer resource
pub fn handle_player_input(
    mut player_input: ResMut<PlayerInput>,
    mut player_mesh_desired_transform: ResMut<PlayerMeshDesiredTransform>,
    mut shoot_timer: ResMut<ShootTimer>,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut lines: ResMut<DebugLines>,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut ev_shoot: EventWriter<ShootEvent>,
    camera_query: Query<(&GlobalTransform, &Camera), With<PrimaryCamera>>,
    player_collider_query: Query<&Transform, With<PlayerCollider>>,
    mut reload_ui_query: Query<&mut Visibility, With<ReloadUI>>,
) {
    let window: &Window = windows.get_primary().unwrap();
    let (camera_transform, camera) = camera_query.single();
    let player_collider_transform = player_collider_query.single();
    let mut reload_ui_visibility = reload_ui_query.single_mut();

    player_mesh_desired_transform.position = player_collider_transform.translation;
    player_mesh_desired_transform.local_up = camera_transform.back();
    player_mesh_desired_transform.local_forward = camera_transform.up();

    // Flag to check if the cursor is inside the window and over the planet collider
    let mut invalid_cursor_pos = false;

    // Raycast parameters
    let max_toi = 600.0;
    let solid = true;
    let filter = QueryFilter::new();

    // If cursor is inside the window
    if let Some(cursor_pos) = window.cursor_position() {
        // Make a raycast from cursor world position parallel to camera direction
        // Could fail when interacting with UI while state gets overwritten
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
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
                    // Show reload UI
                    reload_ui_visibility.is_visible = true;
                    return;
                }

                // Hide the reload UI
                reload_ui_visibility.is_visible = false;

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
            } else {
                invalid_cursor_pos = true;
            }
        } else {
            invalid_cursor_pos = true;
        }
    } else {
        invalid_cursor_pos = true;
    }

    if invalid_cursor_pos {
        if let Some(cursor_pos) = player_input.last_valid_cursor_pos {
            // Cursor position and raycast still needs to be checked because last_valid_cursor_pos won't be valid
            // if the game window's position or size has changed
            if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
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
}

// HELPER FUNCTIONS

// Calculates the tangent in the direction of the vector from the player collider to the hit point on the planet
fn get_tangent_helper(hit_point: Vec3, player_collider_transform: &Transform) -> Vec3 {
    // Get the unit vector in the direction of the vector from the hit point to the player
    let hit_to_player_collider = (hit_point - player_collider_transform.translation).normalize();

    // Cross hit_to_player and player collider's normalized translation vector to get the tangent perpendicular to the desired direction
    let tangent = (hit_to_player_collider.cross(player_collider_transform.translation.normalize()))
        .normalize();

    // Cross again to get the desired direction
    tangent.cross(player_collider_transform.translation.normalize())
}
