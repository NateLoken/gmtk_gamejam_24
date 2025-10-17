use bevy::audio::{AudioBundle, PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;

use crate::components::{
    Ability, CooldownUi, GameOverUI, GameState, GameTimer, GameTimerText, GameUI, HealthText,
    MenuUI, PauseMenu, QuitButton, Resettable, RestartButton, Score, ScoreText, StartButton,
    Wallpaper,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu),
            (cleanup_game_ui, setup_menu.after(cleanup_game_ui)),
        )
        .add_systems(OnExit(GameState::Menu), cleanup_menu)
        .add_systems(
            Update,
            (menu_action_system, quit_action_system).run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            OnEnter(GameState::Running),
            (
                setup_in_game_ui,
                setup_pause_menu.after(setup_in_game_ui),
            ),
        )
        .add_systems(
            OnEnter(GameState::GameOver),
            (cleanup_game_ui, setup_game_over_screen.after(cleanup_game_ui)),
        )
        .add_systems(OnExit(GameState::GameOver), kill_game_over_ui)
        .add_systems(
            Update,
            (restart_action_system, quit_action_system)
                .run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .add_systems(
            Update,
            handle_escape_pressed.run_if(
                in_state(GameState::Running).or_else(in_state(GameState::Paused)),
            ),
        );
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("wallpaper.png"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 3.0)),
            ..Default::default()
        })
        .insert(Wallpaper);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MenuUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Gashadokuro Escape",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 96.0,
                    color: Color::WHITE,
                },
            ));

            parent.spawn(TextBundle::from_section(
                "Defeat the skeleton without being hit to win.",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ));

            parent.spawn(TextBundle::from_section(
                "WASD to move | Q melee | E ranged | T AoE | F dash",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 28.0,
                    color: Color::WHITE,
                },
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|buttons| {
                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(220.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            background_color: Color::srgba(0.25, 0.25, 0.75, 1.0).into(),
                            ..Default::default()
                        })
                        .insert(StartButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Start",
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });

                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(220.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            background_color: Color::srgba(0.75, 0.25, 0.25, 1.0).into(),
                            ..Default::default()
                        })
                        .insert(QuitButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Quit",
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
        });
}

fn cleanup_menu(
    mut commands: Commands,
    menu_nodes: Query<Entity, With<MenuUI>>,
    wallpapers: Query<Entity, With<Wallpaper>>,
) {
    for entity in menu_nodes.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in wallpapers.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_in_game_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_ui: Query<Entity, With<GameUI>>,
) {
    if !existing_ui.is_empty() {
        return;
    }

    commands
        .spawn(
            TextBundle::from_section(
                "WASD to move | Q melee | E ranged | T AoE | F dash",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 24.0,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..Default::default()
            }),
        )
        .insert(GameUI)
        .insert(Resettable);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GameUI)
        .insert(Resettable)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(10.0),
                        left: Val::Px(10.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(GameUI)
                .insert(Resettable)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            "Health: 500",
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 36.0,
                                color: Color::WHITE,
                            },
                        ))
                        .insert(HealthText)
                        .insert(GameUI)
                        .insert(Resettable);

                    parent
                        .spawn(TextBundle::from_section(
                            "Time: 0.0",
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                        ))
                        .insert(GameTimerText)
                        .insert(GameUI)
                        .insert(Resettable);

                    parent
                        .spawn(TextBundle::from_section(
                            "Score: 0",
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                        ))
                        .insert(ScoreText)
                        .insert(GameUI)
                        .insert(Resettable);
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(70.0),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(GameUI)
                .insert(Resettable)
                .with_children(|parent| {
                    for ability in [
                        Ability::Attack,
                        Ability::Ranged,
                        Ability::Dash,
                        Ability::Aoe,
                    ] {
                        let label = match ability {
                            Ability::Attack => "Attack",
                            Ability::Ranged => "Ranged",
                            Ability::Dash => "Dash",
                            Ability::Aoe => "Bladestorm",
                        };

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Percent(20.0),
                                    height: Val::Px(50.0),
                                    margin: UiRect::all(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..Default::default()
                                },
                                background_color: Color::srgba(0.9, 0.9, 0.9, 0.5).into(),
                                ..Default::default()
                            })
                            .insert(GameUI)
                            .insert(Resettable)
                            .with_children(|box_parent| {
                                box_parent
                                    .spawn(TextBundle::from_section(
                                        format!("{label}: 0.0s"),
                                        TextStyle {
                                            font: asset_server.load("FiraSans-Bold.ttf"),
                                            font_size: 28.0,
                                            color: Color::BLACK,
                                        },
                                    ))
                                    .insert(CooldownUi)
                                    .insert(GameUI)
                                    .insert(Resettable);
                            });
                    }
                });
        });
}

fn cleanup_game_ui(mut commands: Commands, query: Query<Entity, With<GameUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_pause_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<PauseMenu>>,
) {
    if !existing.is_empty() {
        return;
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            PauseMenu,
            Resettable,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Game Paused\nPress Esc to Resume\n\nB to return to Menu",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            ));
        });
}

fn show_pause_menu(mut query: Query<&mut Visibility, With<PauseMenu>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}

fn hide_pause_menu(mut query: Query<&mut Visibility, With<PauseMenu>>) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}

fn menu_action_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                menu_sound(&asset_server, &mut commands);
                next_state.set(GameState::Reset);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.75));
            }
        }
    }
}

fn restart_action_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                menu_sound(&asset_server, &mut commands);
                next_state.set(GameState::Reset);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.75));
            }
        }
    }
}

fn quit_action_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
    mut exit: EventWriter<AppExit>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                menu_sound(&asset_server, &mut commands);
                exit.send(AppExit::Success);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.75, 0.35, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.75, 0.25, 0.25));
            }
        }
    }
}

fn handle_escape_pressed(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        menu_sound(&asset_server, &mut commands);
        match current_state.get() {
            GameState::Running => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Running),
            _ => {}
        }
    } else if keyboard_input.just_pressed(KeyCode::KeyB)
        && *current_state.get() == GameState::Paused
    {
        menu_sound(&asset_server, &mut commands);
        next_state.set(GameState::Menu);
    }
}

fn setup_game_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    score: Res<Score>,
    timer: Res<GameTimer>,
) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(24.0),
                ..Default::default()
            },
            background_color: Color::srgba(0.0, 0.0, 0.0, 0.8).into(),
            ..Default::default()
        })
        .insert(GameOverUI)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "You fell to Gashadokuro!",
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 88.0,
                    color: Color::WHITE,
                },
            ));

            parent.spawn(TextBundle::from_section(
                format!("Final Score: {}", score.get_enemies_killed()),
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            ));

            parent.spawn(TextBundle::from_section(
                format!("Time Survived: {:.1} seconds", timer.0),
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 48.0,
                    color: Color::WHITE,
                },
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(16.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|buttons| {
                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(220.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            background_color: Color::srgba(0.25, 0.75, 0.25, 1.0).into(),
                            ..Default::default()
                        })
                        .insert(RestartButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Play Again",
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });

                    buttons
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(220.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            background_color: Color::srgba(0.75, 0.25, 0.25, 1.0).into(),
                            ..Default::default()
                        })
                        .insert(QuitButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Quit",
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
        });
}

fn kill_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn menu_sound(asset_server: &Res<AssetServer>, commands: &mut Commands) {
    commands.spawn(AudioBundle {
        source: asset_server.load("sfx/select.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Once,
            volume: Volume::new(1.3),
            ..Default::default()
        },
    });
}
