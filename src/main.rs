mod components;
mod collision;
mod enemy;
mod player;
mod systems;
mod events;
mod menu;

use bevy::prelude::*;
use collision::CollisionPlugin;
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use events::*;
use components::{GameState, GameTimer, MapGrid, Score};
use systems::*;
use menu::MenuPlugin;

//Assets constants
const PLAYER_SPRITE: &str = "default_guy.png";
const ENEMY_SPRITE: &str = "oni.png";
const LINE_SPRITE: &str = "red_line.png";
const MAP_SPIRITE: &str = "map.png";
const SPRITE_SIZE: (f32, f32) = (225., 225.);
const SPRITE_SCALE: f32 = 0.5;

// Game Cosntants
const BASE_SPEED: f32 = 250.;
const PLAYER_RADIUS: f32 = 500.;

// Enemy Constants
const ENEMY_SPEED: f32 = 150.;

// Resources
#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    enemy: Handle<Image>,
    line: Handle<Image>,
    map: Handle<Image>
}

// Mouse Resource
#[derive(Resource)]
pub struct MouseCoords {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource)]
struct EnemySpawnRate(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((CollisionPlugin, PlayerPlugin, EnemyPlugin, MenuPlugin))
        .insert_resource(Score::new())
        .insert_resource(MapGrid::default())
        .insert_resource(GameTimer(0.0))
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::Reset), reset_game)
        .add_systems(OnEnter(GameState::Running), ensure_base_map)
        .add_systems(
            FixedUpdate,
            (
                clean_dead,
                update_timer.run_if(in_state(GameState::Running)),
                camera_follow_player.run_if(in_state(GameState::Running)),
                update_mouse_position.run_if(in_state(GameState::Running)),
                update_lifetime.run_if(in_state(GameState::Running)),
                update_cooldowns.run_if(in_state(GameState::Running)),
                update_cooldowns_ui.run_if(in_state(GameState::Running)),
                update_ui_text.run_if(in_state(GameState::Running)),
                manage_invulnerability.run_if(in_state(GameState::Running)),
                //flicker_system.run_if(in_state(GameState::Running)),
                check_and_spawn_map.run_if(in_state(GameState::Running)),
                update_bigfoot.run_if(in_state(GameState::Running)),
                //update_player_position.run_if(in_state(GameState::Running)),
                update_bigfoot_position.run_if(in_state(GameState::Running)),
            ))
        .add_event::<CollisionEvent>()
        .run();
}
