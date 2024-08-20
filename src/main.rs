mod components;
mod enemy;
mod player;
mod systems;
mod events;

use bevy::prelude::*;
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use systems::*;
use events::*;
use components::{CurrentGameState, GameState, GameTimer, MapGrid, MousePosition, Points, Score};


//Assets constants
const PLAYER_SPRITE: &str = "default_guy.png";
const ENEMY_SPRITE: &str = "oni.png";
const LINE_SPRITE: &str = "red_line.png";
const MAP_SPIRITE: &str = "map.png";
const SPRITE_SIZE: (f32, f32) = (225., 225.);
const SPRITE_SCALE: f32 = 0.5;

// Game Cosntants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 100.;
const PLAYER_RADIUS: f32 = 500.;

// Enemy Constants
const MAX_ENEMIES: u32 = 1;

// Texture Resource
#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    enemy: Handle<Image>,
    dash: Handle<Image>,
    map: Handle<Image>
}

// Mouse Resource
#[derive(Resource)]
pub struct MouseCoords {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)]
struct EnemyCount(u32);

fn main() {
    App::new()
        // .insert_resource(WindowDescriptor {
        //     title: "Game".to_string(),
        //     ..Default::default()
        // })
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .insert_resource(Score::new())
        .insert_resource(MousePosition::default())
        .insert_resource(Points::default())
        .insert_resource(CurrentGameState { state: GameState::Menu }) 
        .insert_resource(MapGrid::default()) 
        .insert_resource(GameTimer(0.0))
        .init_state::<GameState>()
        //.add_systems(PreStartup, setup_menu)
        .add_systems(Startup, (setup, setup_menu))
        .add_systems(OnExit(GameState::Menu), (kill_wallpaper, despawn_menu, spawn_menu, setup_pause_menu))
        .add_systems(OnEnter(GameState::Menu),(reset_game, kill_game_ui, despawn_menu, setup_menu, reset_game))
        .add_systems(OnEnter(GameState::Reset),(reset_game))
        .add_systems(OnExit(GameState::Reset),(kill_wallpaper, kill_game_over_ui, despawn_menu, spawn_menu, setup_pause_menu))
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_screen)
        //.add_plugin(QuickMenuPlugin::<PauseMenu>::new()) // Add the QuickMenu plugin
        .add_systems(
            FixedUpdate,
            (
                //menu_action_system.run_if(in_state(GameState::Menu)),
                //quit_action_system .run_if(in_state(GameState::Menu)),
                menu_action_system,
                quit_action_system,
                restart_action_system,
                check_won_game,
                update_timer.run_if(in_state(GameState::Running)),
                check_collisions.run_if(in_state(GameState::Running)),
                camera_follow_player.run_if(in_state(GameState::Running)),
                update_mouse_position.run_if(in_state(GameState::Running)),
                update_lifetime.run_if(in_state(GameState::Running)),
                update_cooldowns.run_if(in_state(GameState::Running)),
                update_cooldowns_ui.run_if(in_state(GameState::Running)),
                update_ui_text.run_if(in_state(GameState::Running)),
                manage_invulnerability.run_if(in_state(GameState::Running)),
                flicker_system.run_if(in_state(GameState::Running)),
                check_and_spawn_map.run_if(in_state(GameState::Running)),
                handle_escape_pressed.run_if(in_state(GameState::Running).or_else(in_state(GameState::Paused))),
                update_bigfoot.run_if(in_state(GameState::Running)),
                
                update_player_position.run_if(in_state(GameState::Running)),
                update_bigfoot_position.run_if(in_state(GameState::Running)),
            ))
        .add_event::<CollisionEvent>()
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .run();
}
