use bevy::{color::palettes::css::RED, input_focus::InputFocus, prelude::*};

// Resources
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, States)]
enum MenuState {
    Main,
    Settings,
    #[default]
    Disabled,
}

#[derive(Component)]
enum MenuAction {
    Play,
    Settings,
    BackToMain,
    Quit,
}

// Plugin Setup
pub struct GameMenu;

impl Plugin for GameMenu {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>();
        app.init_state::<MenuState>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, (button_system, menu_action));
    }
}

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

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn((
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

fn button(name: &str, asset_server: &AssetServer, button_type: MenuAction) -> impl Bundle {
    (
        Button,
        Node {
            border: UiRect::all(px(5)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        button_type,
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

fn menu_action(
    interaction_query: Query<(&Interaction, &MenuAction), (Changed<Interaction>)>,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, menu_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_action {
                MenuAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                MenuAction::Play => {
                    menu_state.set(MenuState::Disabled);
                }
                MenuAction::Settings => menu_state.set(MenuState::Settings),
                MenuAction::BackToMain => menu_state.set(MenuState::Main),
            }
        }
    }
}
