use bevy::{
    prelude::{
        default, App, AssetPlugin, ImagePlugin, IntoSystemAppConfig, IntoSystemAppConfigs,
        IntoSystemConfig, IntoSystemConfigs, OnEnter, OnExit, OnUpdate, PluginGroup,
    },
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_prototype_debug_lines::DebugLinesPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_starfield::{GameUnitsToCelestial, StarfieldPlugin};

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
use wgpu::{AddressMode, SamplerDescriptor};

// TODO: add grass to planet
// TODO: cannon ball shooting sfx
// TODO: cannon ball shooting vfx
// TODO: cannon ball explosion sfx
// TODO: cannon ball explosion vfx

fn main() {
    let mut app = App::new();

    // Default plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Loose Cannon".to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            })
            .set(ImagePlugin {
                default_sampler: SamplerDescriptor {
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    ..Default::default()
                },
            }),
    );

    // Third-party plugins
    app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(AtmospherePlugin)
        // .add_plugin(StarfieldPlugin)
        // .add_plugin(DebugLinesPlugin::with_depth_test(true))
        ;

    // Third-party debug plugins
    #[cfg(debug_assertions)]
    app.add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(EditorPlugin::new().in_new_window(Window::default()))
        ;

    // // Custom materials
    // app.add_plugin(MaterialPlugin::<CloudMaterial>::default());

    // Events
    app.add_event::<ShootEvent>();

    // Resources
    app.insert_resource(Score(0))
        .insert_resource(GameUnitsToCelestial {
            origin_latitude: 51.4778,
            origin_longitude: -0.0014,
            ..Default::default()
        });

    // State
    app.add_state::<GameState>();

    // Startup systems
    app.add_startup_system(setup_window);

    // GameState::Playing systems
    app.add_systems(
        (
            setup_scene,
            setup_player,
            setup_player_input,
            setup_asteroids,
            setup_game_ui,
        )
            .chain()
            .in_schedule(OnEnter(GameState::Playing)),
    )
    .add_systems(
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
            .in_set(OnUpdate(GameState::Playing)),
    )
    .add_system(teardown.in_schedule(OnExit(GameState::Playing)));

    // GameState::GameOver systems
    app.add_system(setup_game_over_ui.in_schedule(OnEnter(GameState::GameOver)))
        .add_system(restart_button_system.in_set(OnUpdate(GameState::GameOver)))
        .add_systems(
            (teardown, reset_score)
                .chain()
                .in_schedule(OnExit(GameState::GameOver)),
        );

    // Misc systems
    app.add_system(bevy::window::close_on_esc);

    // Run app
    app.run();
}
