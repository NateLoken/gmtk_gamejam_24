// [x] Main Menu with changable states
// [] Settings Menu
//  [] Controls
//      [] Movement (W,A,S,D)
//      [] Attack (RMB, LMB, Dash, AOE)
//  [x] Sound
//      [x] Display a float with increment buttons on either side
//  May Consider splitting this into a whole ahh module later it's getting a little large

use bevy::{color::palettes::css::RED, input_focus::InputFocus, prelude::*};

use crate::GameState;

// Resources
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Resource)]
struct AudioLevel(f32);

#[derive(Component)]
struct AudioText;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, States)]
enum MenuState {
    Main,
    Settings,
    VolumeSettings,
    ControlSettings,
    #[default]
    Disabled,
}

#[derive(Component)]
enum MenuAction {
    Play,
    Settings,
    Controls,
    Volume,
    VolInc,
    VolDec,
    BackToMain,
    Quit,
}

// Plugin Setup
pub struct GameMenu;

impl Plugin for GameMenu {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.insert_resource(AudioLevel(0.5));
        app.init_state::<MenuState>();
        app.add_systems(OnEnter(GameState::Menu), menu_setup);
        app.add_systems(OnEnter(MenuState::Main), main_menu_setup);
        app.add_systems(OnEnter(MenuState::Settings), settings_menu_setup);
        app.add_systems(OnEnter(MenuState::VolumeSettings), volume_menu_setup);
        app.add_systems(
            Update,
            (menu_action, button_system).run_if(in_state(GameState::Menu)),
        );
    }
}

// Systems
fn button_system(
    mut input_focus: ResMut<InputFocus>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
        ),
        Changed<Interaction>,
    >,
) {
    for (entity, interaction, mut color, mut border_color, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED.into();
                *border_color = BorderColor::all(RED);
                button.set_changed();
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                *color = HOVERED.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                *color = NORMAL_BUTTON.into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn menu_action(
    interaction_query: Query<(&Interaction, &MenuAction), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text, With<AudioText>>,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut audio_level: ResMut<AudioLevel>,
) {
    let mut update_volume = false;
    for (interaction, menu_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_action {
                MenuAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                MenuAction::Play => {
                    menu_state.set(MenuState::Disabled);
                    game_state.set(GameState::Game);
                }
                MenuAction::Settings => menu_state.set(MenuState::Settings),
                MenuAction::Controls => menu_state.set(MenuState::ControlSettings),
                MenuAction::Volume => menu_state.set(MenuState::VolumeSettings),
                MenuAction::VolInc => {
                    *audio_level = AudioLevel((audio_level.0 + 0.05).clamp(0.0, 1.0));
                    audio_level.set_changed();
                    update_volume = true;
                }
                MenuAction::VolDec => {
                    *audio_level = AudioLevel((audio_level.0 - 0.05).clamp(0.0, 1.0));
                    audio_level.set_changed();
                    update_volume = true;
                }
                MenuAction::BackToMain => menu_state.set(MenuState::Main),
            }
        }
    }

    if update_volume && let Ok(mut text) = text_query.single_mut() {
        *text = format!("{:.2}", audio_level.0).into();
        text.set_changed();
    }
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        DespawnOnExit(MenuState::Main),
        Node {
            width: percent(100.),
            height: percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ImageNode::new(assets.load("wallpaper.png")).with_mode(NodeImageMode::Stretch),
        children![(
            Node {
                width: percent(30.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(20.),
                padding: UiRect::all(px(30.)),
                ..Default::default()
            },
            children![
                (
                    Text::new("Agashin"),
                    TextFont {
                        font: assets.load("FiraSans-Bold.ttf"),
                        font_size: 50.0,
                        ..Default::default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::new_with_justify(Justify::Center),
                ),
                button("Play", &assets, MenuAction::Play),
                button("Settings", &assets, MenuAction::Settings),
                button("Exit", &assets, MenuAction::Quit)
            ]
        )],
    ));
}

fn settings_menu_setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((
        DespawnOnExit(MenuState::Settings),
        Node {
            width: percent(100.),
            height: percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ImageNode::new(assets.load("wallpaper.png")).with_mode(NodeImageMode::Stretch),
        children![(
            Node {
                width: percent(30.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(20.),
                padding: UiRect::all(px(30.)),
                ..Default::default()
            },
            children![
                (
                    Text::new("Settings"),
                    TextFont {
                        font: assets.load("FiraSans-Bold.ttf"),
                        font_size: 50.0,
                        ..Default::default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::new_with_justify(Justify::Center),
                ),
                button("Controls", &assets, MenuAction::Controls),
                button("Volume", &assets, MenuAction::Volume),
                button("Back", &assets, MenuAction::BackToMain)
            ]
        )],
    ));
}

fn volume_menu_setup(mut commands: Commands, assets: Res<AssetServer>, volume: Res<AudioLevel>) {
    commands.spawn((
        DespawnOnExit(MenuState::VolumeSettings),
        Node {
            width: percent(100.),
            height: percent(100.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        ImageNode::new(assets.load("wallpaper.png")).with_mode(NodeImageMode::Stretch),
        children![(
            Node {
                width: percent(30.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
                row_gap: px(20.),
                padding: UiRect::all(px(30.)),
                ..Default::default()
            },
            children![
                (
                    Text::new("Volume"),
                    TextFont {
                        font: assets.load("FiraSans-Bold.ttf"),
                        font_size: 50.0,
                        ..Default::default()
                    },
                    TextColor(Color::WHITE),
                    TextLayout::new_with_justify(Justify::Center),
                ),
                (
                    Node {
                        border: UiRect::all(px(5)),
                        width: percent(100.),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        column_gap: px(20.),
                        ..Default::default()
                    },
                    BorderRadius::all(px(15.)),
                    BackgroundColor(NORMAL_BUTTON),
                    children![
                        button("<", &assets, MenuAction::VolDec),
                        (
                            Text::new(format!("{:.2}", volume.0)),
                            TextFont {
                                font: assets.load("FiraSans-Bold.ttf"),
                                font_size: 50.0,
                                ..Default::default()
                            },
                            TextColor(Color::WHITE),
                            TextLayout::new_with_justify(Justify::Center),
                            AudioText,
                        ),
                        button(">", &assets, MenuAction::VolInc),
                    ]
                ),
                button("Back", &assets, MenuAction::Settings),
            ]
        )],
    ));
}

fn button(name: &str, asset_server: &AssetServer, button_type: MenuAction) -> impl Bundle {
    (
        Button,
        button_type,
        Node {
            border: UiRect::all(px(5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        BorderColor::all(Color::WHITE),
        BorderRadius::all(px(15.)),
        BackgroundColor(Color::BLACK),
        children![(
            Text::new(name),
            TextFont {
                font: asset_server.load("FiraSans-Bold.ttf"),
                font_size: 33.0,
                ..Default::default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}
