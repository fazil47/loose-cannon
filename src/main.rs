use bevy::{
    prelude::{
        default, in_state, App, IntoSystemConfigs, OnEnter, OnExit, PluginGroup, Startup, Update,
    },
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
// use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
// use bevy_starfield::{GameUnitsToCelestial, StarfieldPlugin};

#[cfg(debug_assertions)]
use bevy_editor_pls::prelude::EditorPlugin;
#[cfg(debug_assertions)]
use bevy_rapier3d::render::RapierDebugRenderPlugin;

use loose_cannon::{
    asteroids::{setup_asteroids, spawn_asteroids},
    cannon_ball::shoot_cannon_ball,
    common::{
        gravity, handle_collisions, move_camera, reset_score, setup_scene, setup_window, teardown,
        GameState, Score,
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
    let mut app = App::new();

    // Default plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Loose Cannon".to_string(),
            ..default()
        }),
        ..default()
    }));

    // Third-party plugins
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(AtmospherePlugin)
        // .add_plugin(StarfieldPlugin)
        // .add_plugin(DebugLinesPlugin::with_depth_test(true))
        ;

    // Third-party debug plugins
    #[cfg(debug_assertions)]
    app.add_plugins((
        RapierDebugRenderPlugin::default(),
        EditorPlugin::new().in_new_window(Window::default()),
    ));

    // // Custom materials
    // app.add_plugin(MaterialPlugin::<CloudMaterial>::default());

    // Events
    app.add_event::<ShootEvent>();

    // Resources
    app.insert_resource(Score(0));

    // State
    app.add_state::<GameState>();

    // Startup systems
    app.add_systems(Startup, setup_window);

    // GameState::Playing systems
    app.add_systems(
        OnEnter(GameState::Playing),
        (
            setup_scene,
            setup_player,
            setup_player_input,
            setup_asteroids,
            setup_game_ui,
        )
            .chain(),
    )
    .add_systems(
        Update,
        (
            gravity,
            handle_player_input,
            set_player_mesh_transform,
            apply_player_collider_impulse,
            shoot_cannon_ball,
            move_camera,
            handle_collisions,
            spawn_asteroids,
            update_score_ui,
        )
            .chain()
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(OnExit(GameState::Playing), teardown);

    // GameState::GameOver systems
    app.add_systems(OnEnter(GameState::GameOver), setup_game_over_ui)
        .add_systems(
            Update,
            (restart_button_system).run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnExit(GameState::GameOver), (teardown, reset_score).chain());

    // Misc systems
    app.add_systems(Update, bevy::window::close_on_esc);

    // Run app
    app.run();
}
