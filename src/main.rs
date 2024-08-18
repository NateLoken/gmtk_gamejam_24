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
use components::{CurrentGameState, GameState, MousePosition, Points, Score};


//Assets constants
const PLAYER_SPRITE: &str = "blue_box.png";
const ENEMY_SPRITE: &str = "pink_box.png";
const LINE_SPRITE: &str = "red_line.png";
const SPRITE_SIZE: (f32, f32) = (225., 225.);
const SPRITE_SCALE: f32 = 0.5;

// Game Cosntants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 100.;
const PLAYER_RADIUS: f32 = 500.;

// Enemy Constants
const MAX_ENEMIES: u32 = 5;

// Texture Resource
#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    enemy: Handle<Image>,
    dash: Handle<Image>,
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
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .insert_resource(Score::new())
        .insert_resource(MousePosition::default())
        .insert_resource(Points::default())
        .insert_resource(CurrentGameState { state: GameState::Running }) 
        .init_state::<GameState>()
        .add_systems(Startup, (setup, setup_pause_menu))
        //.add_plugin(QuickMenuPlugin::<PauseMenu>::new()) // Add the QuickMenu plugin
        .add_systems(
            FixedUpdate,
            (
                display_score.run_if(in_state(GameState::Running)),
                check_collisions.run_if(in_state(GameState::Running)),
                camera_follow_player.run_if(in_state(GameState::Running)),
                update_mouse_position.run_if(in_state(GameState::Running)),
                update_lifetime.run_if(in_state(GameState::Running)),
                update_cooldowns.run_if(in_state(GameState::Running)),
                update_cooldowns_ui.run_if(in_state(GameState::Running)),
                update_ui_text.run_if(in_state(GameState::Running)),
                manage_invulnerability.run_if(in_state(GameState::Running)),
                flicker_system.run_if(in_state(GameState::Running)),
                handle_escape_pressed.run_if(in_state(GameState::Running).or_else(in_state(GameState::Paused))),
            ))
        .add_event::<CollisionEvent>()
        .add_systems(OnEnter(GameState::Paused), show_pause_menu)
        .add_systems(OnExit(GameState::Paused), hide_pause_menu)
        .run();
}
