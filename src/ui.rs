use bevy::{
    prelude::{
        AssetServer, BuildChildren, ButtonBundle, Color, Commands, Name, NodeBundle, Res,
        TextBundle,
    },
    text::TextStyle,
    ui::{AlignItems, FlexDirection, JustifyContent, Size, Style, UiRect, Val},
    utils::default,
};

// STARTUP SYSTEMS

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI root node
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("UI_Root"))
        .with_children(|parent| {
            // It's children are two nodes
            // One containing the score and the reload time indicator
            // The other containing the game over text and restart button

            // Score and reload time indicator - In Game UI
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(20.0)),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
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
                        .insert(Name::new("Score_Indicator"));

                    // Reload time indicator
                    parent
                        .spawn(
                            TextBundle::from_section(
                                "Reload Timer",
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
                        .insert(Name::new("Reload_Timer_Indicator"));
                });

            // Game over text and restart button - Game Over UI
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(70.0), Val::Percent(70.0)),
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
        });
}

// SYSTEMS

pub fn update_score() {
    todo!("System to update score text");
}

pub fn show_game_over() {
    todo!("System to show game over menu by checking value of game state. The menu must contain a button to restart the game.")
}
