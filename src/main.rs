use bevy::prelude::{default, App, MaterialPlugin, PluginGroup};
use bevy::window::{WindowDescriptor, WindowPlugin};
use bevy::DefaultPlugins;
use bevy_editor_pls::prelude::EditorPlugin;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};

use bevy_rapier3d::render::RapierDebugRenderPlugin;
use loose_cannon::common::setup_window;
use loose_cannon::ui::setup_ui;
use loose_cannon::{
    cannon_ball::shoot_cannon_ball,
    common::{gravity, handle_collisions, move_camera},
    cubemap::{construct_skybox, CubemapMaterial},
    input::{handle_player_input, ShootEvent},
    player::{apply_player_collider_impulse, set_player_mesh_transform},
    setup::setup,
};

// TODO: game over ui
// TODO: add a single type of enemy
// TODO: enemies should spawn from reasonably spaced random points
// TODO: count score (in proportion to number of enemies killed)
// TODO: show score in ui
// TODO: player rotation should be smooth
// TODO: cannon ball explosion vfx
// TODO: cannon ball shooting vfx
// TODO: cannon ball explosion sfx
// TODO: cannon ball shooting sfx
// TODO: add grass to planet
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
        .add_startup_system(setup)
        .add_startup_system(setup_window)
        .add_startup_system(setup_ui)
        .add_system(gravity)
        .add_system(handle_player_input)
        .add_system(set_player_mesh_transform)
        .add_system(apply_player_collider_impulse)
        .add_system(shoot_cannon_ball)
        .add_system(move_camera)
        .add_system(handle_collisions)
        .add_system(construct_skybox)
        .add_event::<ShootEvent>()
        .run();
}
