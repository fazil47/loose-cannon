use bevy::{
    prelude::{
        AssetServer, BuildChildren, ButtonBundle, Camera3dBundle, Changed, Color, Commands,
        Component, Name, NodeBundle, Query, Res, ResMut, State, TextBundle, Transform, Visibility,
        With, Camera,
    },
    text::{Text, TextStyle},
    ui::{
        AlignItems, BackgroundColor, FlexDirection, Interaction, JustifyContent, Size, Style,
        UiRect, Val,
    },
    utils::default,
};

use crate::common::{GameState, Score};

// CONSTANTS

const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);
const HOVERED_BUTTON: Color = Color::rgb(0.9, 0.9, 0.9);
const PRESSED_BUTTON: Color = Color::rgb(0.8, 0.8, 0.8);

// COMPONENTS

#[derive(Component)]
pub struct ScoreUI {}

#[derive(Component)]
pub struct ReloadUI {}

#[derive(Component)]
pub struct RestartButton {}

// STARTUP SYSTEMS

pub fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // In Game UI
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("In_Game_UI"))
        .with_children(|parent| {
            // Score
            parent
                .spawn(
                    TextBundle::from_section(
                        "Score: 0",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        flex_shrink: 1.0,
                        ..default()
                    }),
                )
                .insert(Name::new("Score_Indicator"))
                .insert(ScoreUI {});

            // Reload indicator
            parent
                .spawn(
                    TextBundle::from_section(
                        "Reloading...",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(5.0)),
                        flex_shrink: 1.0,
                        ..default()
                    }),
                )
                .insert(Name::new("Reload_Indicator"))
                .insert(ReloadUI {})
                .insert(Visibility::INVISIBLE);
        });
}

pub fn setup_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
) {
    // Camera for UI
    commands.spawn((Camera3dBundle {
        camera: Camera {
            priority: 5,
            ..default()
        },
        transform: Transform::default(),
        ..default()
    },));

    // Game over text and restart button - Game Over UI
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgb(0.25, 0.25, 0.25).into(),
            ..default()
        })
        .insert(Name::new("Game_Over_UI"))
        .with_children(|parent| {
            // Game over text
            parent
                .spawn(
                    TextBundle::from_section(
                        "Game Over",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    }),
                )
                .insert(Name::new("Game_Over_Text"));

            // Final score
            parent
                .spawn(
                    TextBundle::from_section(
                        format!("Final Score: {}", score.0),
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 50.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    }),
                )
                .insert(Name::new("Game_Over_Text"));

            // Restart button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(50.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    ..default()
                })
                .insert(Name::new("Restart_Button"))
                .insert(RestartButton {})
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Restart",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 30.0,
                            color: Color::BLACK,
                        },
                    ));
                });
        });
}

// SYSTEMS

// This system runs only when state is set to Playing
pub fn update_score_ui(score: Res<Score>, mut score_ui_query: Query<&mut Text, With<ScoreUI>>) {
    let mut score_ui = score_ui_query.single_mut();

    score_ui.sections[0].value = format!("Score: {}", score.0);
}

// This system runs only when state is set to GameOver
pub fn restart_button_system(
    mut game_state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                game_state.overwrite_replace(GameState::Playing).unwrap();
            }
        }
    }
}
