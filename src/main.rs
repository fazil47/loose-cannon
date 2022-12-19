use bevy::{
    prelude::{default, App, AssetPlugin, MaterialPlugin, PluginGroup, SystemSet},
    window::{WindowDescriptor, WindowPlugin},
    DefaultPlugins,
};
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_editor_pls::prelude::EditorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::{
    prelude::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};

use loose_cannon::{
    asteroid::{setup_asteroid, spawn_asteroid},
    cannon_ball::shoot_cannon_ball,
    clouds::{setup_clouds, update_clouds, CloudMaterial},
    common::{
        gravity, handle_collisions, move_camera, reset_rapier, reset_score, setup_scene,
        setup_window, teardown, GameState, Score,
    },
    input::{handle_player_input, setup_player_input, ShootEvent},
    player::{apply_player_collider_impulse, set_player_mesh_transform, setup_player},
    ui::{restart_button_system, setup_game_over_ui, setup_game_ui, update_score_ui},
};

// TODO: add grass to planet
// TODO: cannon ball shooting sfx
// TODO: cannon ball shooting vfx
// TODO: cannon ball explosion sfx
// TODO: cannon ball explosion vfx

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Loose Cannon".to_string(),
                        ..default()
                    },
                    ..default()
                })
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                }),
        )
        .add_plugin(MaterialPlugin::<CloudMaterial>::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(AtmospherePlugin)
        .add_plugin(EditorPlugin)
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        .add_state(GameState::Playing)
        .add_startup_system(setup_window)
        .add_system_set(
            SystemSet::on_enter(GameState::Playing)
                .with_system(setup_scene)
                .with_system(setup_player)
                // .with_system(setup_cubemap)
                .with_system(setup_player_input)
                .with_system(setup_asteroid)
                .with_system(setup_clouds)
                .with_system(setup_game_ui)
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
                .with_system(spawn_asteroid)
                .with_system(update_clouds)
                .with_system(update_score_ui), // .with_system(construct_cubemap),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(teardown))
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(setup_game_over_ui))
        .add_system_set(
            SystemSet::on_update(GameState::GameOver).with_system(restart_button_system),
        )
        .add_system_set(
            SystemSet::on_exit(GameState::GameOver)
                .with_system(teardown)
                .with_system(reset_score),
        )
        .add_system(bevy::window::close_on_esc)
        .insert_resource(Score(0))
        .add_event::<ShootEvent>()
        .run();
}
