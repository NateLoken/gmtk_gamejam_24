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

//Assets constants
const PLAYER_SPRITE: &str = "blue_box.png";
const ENEMY_SPRITE: &str = "pink_box.png";
const SPRITE_SIZE: (f32, f32) = (225., 225.);
const SPRITE_SCALE: f32 = 0.5;

// Game Cosntants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;
const PLAYER_RADIUS: f32 = 500.;

// Enemy Constants
const MAX_ENEMIES: u32 = 1;

// Texture Resource
#[derive(Resource)]
struct GameTextures {
    player: Handle<Image>,
    enemy: Handle<Image>,
}

#[derive(Resource)]
struct EnemyCount(u32);

fn main() {
    App::new()
        .insert_resource(Score::new())
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                camera_follow_player,
                display_score,
                check_collisions,
                handle_collisions,
            ),
        )
        .add_event::<CollisionEvent>()
        .run();
}
