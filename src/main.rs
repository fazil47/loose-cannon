use bevy::prelude::{default, App, MaterialPlugin, PluginGroup, SystemSet};
use bevy::window::{WindowDescriptor, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::EditorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};

use bevy_rapier3d::render::RapierDebugRenderPlugin;
use loose_cannon::common::{setup_window, teardown, GameState};
use loose_cannon::setup::setup_common;
use loose_cannon::ui::{restart_button_system, setup_game_over_ui, setup_ui};
use loose_cannon::{
    cannon_ball::shoot_cannon_ball,
    common::{gravity, handle_collisions, move_camera, reset_rapier},
    cubemap::{construct_skybox, CubemapMaterial},
    input::{handle_player_input, ShootEvent},
    player::{apply_player_collider_impulse, set_player_mesh_transform},
    setup::setup_game,
};

// TODO: add a single type of enemy
// -> enemies spawn at random locations which are not too close to the player or other enemies
// -> enemies move towards the player
// -> enemies despawn when they get hit by a cannon ball
// -> game ends when an enemy reaches the player
// TODO: count score (in proportion to number of enemies killed)
// TODO: show score in ui
// TODO: cannon ball shooting sfx
// TODO: cannon ball shooting vfx
// TODO: there should be a delay before the game gets over after the player dies
// TODO: cannon ball explosion sfx
// TODO: cannon ball explosion vfx
// TODO: add grass to planet
// TODO: player rotation should be smooth
// TODO: add trees to planet
// TODO: add mesh for enemy spawn point

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Loose Cannon".to_string(),
                ..default()
            },
            ..default()
        }))
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(EditorPlugin)
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_state(GameState::Playing)
        .add_startup_system(setup_window)
        .add_startup_system(setup_common)
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(setup_game)
                .with_system(setup_ui)
                .with_system(reset_rapier),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(gravity)
                .with_system(handle_player_input)
                .with_system(set_player_mesh_transform)
                .with_system(apply_player_collider_impulse)
                .with_system(shoot_cannon_ball)
                .with_system(move_camera)
                .with_system(handle_collisions)
                .with_system(construct_skybox),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown))
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(setup_game_over_ui))
        .add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(restart_button_system),
        )
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(teardown))
        .add_system(bevy::window::close_on_esc)
        .add_event::<ShootEvent>()
        .run();
}
