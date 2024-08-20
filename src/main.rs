mod components;
mod collision;
mod enemy;
mod player;
mod systems;
mod events;

use bevy::prelude::*;
use collision::CollisionPlugin;
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use systems::*;
use events::*;

//Assets constants
const PLAYER_SPRITE: &str = "blue_box.png";
const ENEMY_SPRITE: &str = "pink_box.png";
const BOSS_SPRITE: &str = "red_box.png";
const LINE_SPRITE: &str = "red_line.png";
const SPRITE_SIZE: (f32, f32) = (225., 225.);
const SPRITE_SCALE: f32 = 0.5;

// Game Cosntants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 250.;
const PLAYER_RADIUS: f32 = 500.;

// Enemy Constants
const MAX_ENEMIES: u32 = 10;
const ENEMY_SPEED: f32 = 150.;

// Resources
#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    enemy: Handle<Image>,
    boss: Handle<Image>,
    line: Handle<Image>,
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
        .add_plugins(CollisionPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                camera_follow_player,
                display_score,
                camera_follow_player,
                clean_dead,
                update_mouse_position,
                update_lifetime,
                update_cooldowns, // Manage ability cooldowns
                manage_invulnerability,
            ),
        )
        .add_event::<CollisionEvent>()
        .run();
}
